//! Whisper に渡す認識コンテキストの構築
//!
//! `initial_prompt` にはトークン数の上限があり、超過分は先頭から切り捨てられる。
//! 重要な語ほど末尾に置き、予算を超える分は先頭から落とす。

/// `initial_prompt` に渡せるトークン数の上限（`n_text_ctx` / 2）。
pub const PROMPT_TOKEN_LIMIT: usize = 224;

/// 語の区切り。読点は日本語の列挙として自然で、かつトークン消費が小さい。
const SEPARATOR: &str = "、";

/// 語彙をトークン予算に収まるプロンプトへ組み立てる。
///
/// `groups` は重要度の高い順に並べる。Whisper は予算を超えたプロンプトの先頭を
/// 切り捨てるため、重要度の高いグループほど末尾へ配置する。予算に収まらない分は
/// 重要度の低い語から順に落とす。
///
/// `count_tokens` は文字列のトークン数を返す。Whisper のトークナイザは
/// モデルに紐づくため、呼び出し側から渡す。
pub fn build_prompt(
    groups: &[&[&str]],
    budget: usize,
    count_tokens: impl Fn(&str) -> usize,
) -> String {
    let words: Vec<&str> = groups
        .iter()
        .rev()
        .flat_map(|g| g.iter().copied())
        .collect();

    // 先頭ほど重要度が低い。予算に収まるまで先頭を削っていく。
    let mut start = 0;
    while start < words.len() {
        let prompt = words[start..].join(SEPARATOR);
        if count_tokens(&prompt) <= budget {
            return prompt;
        }
        start += 1;
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 文字数をトークン数とみなす。予算ロジックの検証にはこれで足りる。
    fn count_chars(text: &str) -> usize {
        text.chars().count()
    }

    #[test]
    fn orders_groups_from_least_to_most_important() {
        let groups: [&[&str]; 2] = [&["プレイヤーA"], &["カフェテリア", "リアクター"]];

        let prompt = build_prompt(&groups, 1000, count_chars);

        assert_eq!(prompt, "カフェテリア、リアクター、プレイヤーA");
    }

    #[test]
    fn drops_least_important_words_when_over_budget() {
        let groups: [&[&str]; 2] = [&["プレイヤーA"], &["カフェテリア", "リアクター"]];

        // "リアクター、プレイヤーA" = 12文字。"カフェテリア、" を足すと19文字で超える。
        let prompt = build_prompt(&groups, 12, count_chars);

        assert_eq!(prompt, "リアクター、プレイヤーA");
    }
}

/// 現行プロンプトの構成要素ごとのトークン数を計測する。
/// モデルの読み込みを伴うため通常のテスト実行からは除外している。
/// 実行: cargo test --lib -- --ignored --nocapture measure_prompt
#[cfg(test)]
mod measurement {
    use super::*;
    use crate::i18n::words::ja::JapaneseWords;
    use crate::resources::whisper_transcriber::WhisperModel;
    use whisper_rs::{WhisperContext, WhisperContextParameters};

    #[test]
    #[ignore]
    fn measure_prompt_token_budget() {
        let ctx = WhisperContext::new_with_params(
            WhisperModel::SMALL.path(),
            WhisperContextParameters::default(),
        )
        .expect("モデルの読み込みに失敗");
        let count = |text: &str| ctx.tokenize(text, 4096).expect("tokenizeに失敗").len();

        let terms = [
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
        .join("、");

        let players = (1..=10)
            .map(|i| format!("プレイヤー{}(あか)", i))
            .collect::<Vec<_>>()
            .join("、");

        let maps: [(&str, &[&str]); 4] = [
            ("Skeld", JapaneseWords::SKELD_ROOMS),
            ("Mira", JapaneseWords::MIRA_ROOMS),
            ("Polus", JapaneseWords::POLUS_ROOMS),
            ("Airship", JapaneseWords::AIRSHIP_ROOMS),
        ];

        println!("\n--- 構成要素ごとのトークン数 (上限 {PROMPT_TOKEN_LIMIT}) ---");
        println!("用語リスト         : {:>4}", count(&terms));
        println!("プレイヤー名10人分 : {:>4}", count(&players));
        for (name, rooms) in maps {
            println!("部屋名 {name:<8} : {:>4}", count(&rooms.join("、")));
        }

        println!("\n--- マップごとの現行プロンプト全体 ---");
        for (name, rooms) in maps {
            let prompt = format!(
                "Among Usのゲーム。プレイヤー: {}。部屋: {}。用語: {}。",
                players,
                rooms.join("、"),
                terms
            );
            let total = count(&prompt);
            println!(
                "{name:<8}: {total:>4} トークン  超過 {:>4}",
                total.saturating_sub(PROMPT_TOKEN_LIMIT)
            );
        }
        println!();
    }

    /// 語彙の表現を変えたときのトークン数を比較する。予算配分の判断材料。
    /// 実行: cargo test --lib -- --ignored --nocapture compare_prompt
    #[test]
    #[ignore]
    fn compare_prompt_variants() {
        let ctx = WhisperContext::new_with_params(
            WhisperModel::SMALL.path(),
            WhisperContextParameters::default(),
        )
        .expect("モデルの読み込みに失敗");
        let count = |text: &str| ctx.tokenize(text, 4096).expect("tokenizeに失敗").len();

        let is_ascii_only = |s: &&str| s.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ');

        let maps: [(&str, &[&str]); 4] = [
            ("Skeld", JapaneseWords::SKELD_ROOMS),
            ("Mira", JapaneseWords::MIRA_ROOMS),
            ("Polus", JapaneseWords::POLUS_ROOMS),
            ("Airship", JapaneseWords::AIRSHIP_ROOMS),
        ];

        println!("\n--- 部屋名: 全部 vs 日本語のみ ---");
        for (name, rooms) in maps {
            let all = count(&rooms.join("、"));
            let ja: Vec<&str> = rooms
                .iter()
                .copied()
                .filter(|s| !is_ascii_only(s))
                .collect();
            println!(
                "{name:<8}: 全 {all:>4} → 日本語のみ {:>4} ({}語)",
                count(&ja.join("、")),
                ja.len()
            );
        }

        println!("\n--- プレイヤー名: 色付き vs 名前のみ ---");
        for label in ["プレイヤー", "ゆう", "しろねこ2号"] {
            let with_color = (1..=10)
                .map(|i| format!("{label}{i}(あか)"))
                .collect::<Vec<_>>()
                .join("、");
            let bare = (1..=10)
                .map(|i| format!("{label}{i}"))
                .collect::<Vec<_>>()
                .join("、");
            println!(
                "{label:<12}: 色付き {:>4} → 名前のみ {:>4}",
                count(&with_color),
                count(&bare)
            );
        }
    }
}
