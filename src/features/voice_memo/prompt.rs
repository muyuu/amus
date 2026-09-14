//! 認識コンテキスト（Whisper の initial_prompt）の組み立て
//!
//! プレイヤー名は最も認識させたい語であり、かつモデルが最も苦手とする固有名詞でもある。
//! 予算超過時に切り捨てられないよう、重要度の低い部屋名から落とす。

use crate::i18n::words::ja::JapaneseWords;
use crate::resources::whisper_prompt::build_prompt;

/// 認識コンテキストの元になる語彙。
///
/// 組み立て直しはトークナイザを回すため、変化したときだけ行えるよう比較可能にしている。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Vocabulary {
    /// 空の名前は除いておくこと。
    pub player_names: Vec<String>,
    /// 現在のマップの部屋名。英語表記が混ざっていてよい。
    pub room_names: &'static [&'static str],
}

impl Vocabulary {
    /// 語彙補正で正規表記として使う語。
    ///
    /// プロンプトと違いトークン予算が無いため、優先度で削らず全て対象にする。
    /// 英語表記は日本語の発話から当たらず、当たっても出力として望ましくないので除く。
    pub(super) fn known_terms(&self) -> Vec<&str> {
        self.player_names
            .iter()
            .map(String::as_str)
            .chain(game_terms())
            .chain(self.room_names.iter().copied())
            .filter(|name| !is_english_alias(name))
            .collect()
    }
}

/// 認識コンテキストを組み立てる。
///
/// `budget` はモデルごとの安全なトークン上限（[`crate::resources::whisper_transcriber::WhisperModel::prompt_token_budget`]）。
/// `count_tokens` は文字列のトークン数を返す。Whisper のトークナイザはモデルに
/// 紐づくため呼び出し側から渡す。
///
/// 予算に収まらない場合は部屋名から落とす。プレイヤー名と用語は残る。
pub(super) fn build_recognition_context(
    vocabulary: &Vocabulary,
    budget: usize,
    count_tokens: impl Fn(&str) -> usize,
) -> String {
    let players: Vec<&str> = vocabulary.player_names.iter().map(String::as_str).collect();
    let terms = game_terms();
    let rooms: Vec<&str> = vocabulary
        .room_names
        .iter()
        .copied()
        .filter(|name| !is_english_alias(name))
        .collect();

    // 重要度の高い順。build_prompt が末尾へ並べ替え、溢れた分を先頭から落とす。
    //
    // トリガーワードはプレイヤー名に次ぐ。プロンプトに載っていない語はモデルがまず
    // 出さず、出なければターン境界が決まらない。数語なので予算も脅かさない。
    let groups: [&[&str]; 4] = [&players, super::hotword::PROMPT_WORDS, &terms, &rooms];

    build_prompt(&groups, budget, count_tokens)
}

/// 部屋名の英語表記かどうか。
///
/// 日本語音声にはまず現れないため予算を割く価値がない。聞き取り結果の英語表記への
/// 対応は語彙補正側で行う。
fn is_english_alias(name: &str) -> bool {
    name.chars().all(|c| c.is_ascii_alphabetic() || c == ' ')
}

/// Among Us 固有の用語。
fn game_terms() -> [&'static str; 10] {
    [
        JapaneseWords::SABOTAGE,
        JapaneseWords::SABOTAGE_COMMS,
        JapaneseWords::SABOTAGE_LIGHTS,
        JapaneseWords::SABOTAGE_O2,
        JapaneseWords::SABOTAGE_REACTOR,
        JapaneseWords::SABOTAGE_DOORS,
        JapaneseWords::SPAWN,
        JapaneseWords::VENT,
        JapaneseWords::TASK,
        JapaneseWords::DISCUSSION,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_chars(text: &str) -> usize {
        text.chars().count()
    }

    fn vocabulary(players: &[&str], rooms: &'static [&'static str]) -> Vocabulary {
        Vocabulary {
            player_names: players.iter().map(|s| s.to_string()).collect(),
            room_names: rooms,
        }
    }

    /// 予算を気にしないテストで使う、十分に大きい値。
    const AMPLE_BUDGET: usize = 1000;

    #[test]
    fn carries_the_trigger_words() {
        let vocab = vocabulary(&["ゆう"], &["カフェテリア"]);

        let prompt = build_recognition_context(&vocab, AMPLE_BUDGET, count_chars);

        // トリガーワードが出力に現れないと検知が成立しない
        assert!(prompt.contains("ターン開始"), "prompt={}", prompt);
        assert!(prompt.contains("ターン終了"), "prompt={}", prompt);
    }

    #[test]
    fn places_player_names_last() {
        let vocab = vocabulary(&["ゆう"], &["カフェテリア"]);

        let prompt = build_recognition_context(&vocab, AMPLE_BUDGET, count_chars);

        assert!(
            prompt.ends_with("ゆう"),
            "プレイヤー名が末尾にない: {prompt}"
        );
        assert!(prompt.starts_with("カフェテリア"), "実際: {prompt}");
    }

    #[test]
    fn drops_english_room_aliases() {
        let vocab = vocabulary(&[], &["カフェテリア", "Cafeteria", "リアクター", "Reactor"]);

        let prompt = build_recognition_context(&vocab, AMPLE_BUDGET, count_chars);

        assert!(prompt.contains("カフェテリア"), "実際: {prompt}");
        assert!(
            !prompt.contains("Cafeteria"),
            "英語表記が残っている: {prompt}"
        );
    }

    #[test]
    fn keeps_player_names_when_rooms_do_not_fit() {
        let vocab = vocabulary(&["ゆう"], &["カフェテリア", "リアクター"]);
        // 予算を用語とプレイヤー名だけで埋まる程度に絞る。
        let budget = game_terms().join("、").chars().count() + "、ゆう".chars().count();

        let prompt = build_recognition_context(&vocab, budget, count_chars);

        assert!(prompt.ends_with("ゆう"), "プレイヤー名が落ちた: {prompt}");
        assert!(
            !prompt.contains("カフェテリア"),
            "部屋名が落ちていない: {prompt}"
        );
    }
}

/// 実際のモデルで組み立て結果を確認する。
/// 実行: cargo test --lib -- --ignored --nocapture verify_context
#[cfg(test)]
mod verification {
    use super::*;
    use crate::models::Area;
    use crate::resources::whisper_transcriber::WhisperTranscriber;
    use crate::resources::TranscribeSetup;

    #[test]
    #[ignore]
    fn verify_context_keeps_player_names() {
        let transcriber =
            WhisperTranscriber::new(TranscribeSetup::CPU).expect("モデルの読み込みに失敗");

        let player_names: Vec<String> = ["あおい", "ゆう", "しろねこ", "たろう", "はなこ"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let maps = [
            ("Skeld", Area::Skeld),
            ("Mira", Area::Mira),
            ("Polus", Area::Polus),
            ("Airship", Area::AirShip),
        ];

        for (label, area) in maps {
            let rooms: &'static [&'static str] = match area {
                Area::Skeld => JapaneseWords::SKELD_ROOMS,
                Area::Mira => JapaneseWords::MIRA_ROOMS,
                Area::Polus => JapaneseWords::POLUS_ROOMS,
                Area::AirShip => JapaneseWords::AIRSHIP_ROOMS,
            };
            let vocabulary = Vocabulary {
                player_names: player_names.clone(),
                room_names: rooms,
            };

            let budget = transcriber.setup().model.prompt_token_budget();
            let prompt =
                build_recognition_context(&vocabulary, budget, |t| transcriber.count_tokens(t));
            let tokens = transcriber.count_tokens(&prompt);
            let kept_rooms = rooms.iter().filter(|r| prompt.contains(**r)).count();

            println!(
                "\n[{label}] {tokens} トークン / 部屋名 {kept_rooms}/{} 残存",
                rooms.len()
            );
            for name in &player_names {
                assert!(prompt.contains(name), "[{label}] {name} が落ちた");
            }
            println!("  プレイヤー名: 全{}名とも残存", player_names.len());
            println!("  {prompt}");
        }
    }
}
