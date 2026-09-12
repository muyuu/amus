//! トリガーワードの照合
//!
//! 書き起こしたテキストにトリガーワードが含まれるかを判定する。音声認識の結果は
//! 表記が揺れるため、読みへ正規化した上で、多少の誤りを許して照合する。

/// 検出したトリガーワードの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hotword {
    TurnStart,
    TurnEnd,
}

/// トリガーワードの照合器。
///
/// 1つのトリガーワードに複数の表記を登録できる。漢字の読みは求められないため
/// （「ターン開始」と「ターンかいし」は別物として扱われる）、想定する表記を並べて登録する。
pub(super) struct HotwordMatcher {
    start_forms: Vec<String>,
    end_forms: Vec<String>,
}

impl HotwordMatcher {
    pub(super) fn new(start_words: &[&str], end_words: &[&str]) -> Self {
        Self {
            start_forms: start_words.iter().map(|w| normalize(w)).collect(),
            end_forms: end_words.iter().map(|w| normalize(w)).collect(),
        }
    }

    /// テキストにトリガーワードが含まれていればその種類を返す。
    pub(super) fn find(&self, text: &str) -> Option<Hotword> {
        let text: Vec<char> = normalize(text).chars().collect();

        // 開始と終了が同時に含まれることは想定しない。先に開始を見る。
        if self
            .start_forms
            .iter()
            .any(|form| contains_close(&text, form))
        {
            return Some(Hotword::TurnStart);
        }
        if self
            .end_forms
            .iter()
            .any(|form| contains_close(&text, form))
        {
            return Some(Hotword::TurnEnd);
        }
        None
    }
}

/// テキストのどこかに、そのトリガーワードと十分近い並びがあるか。
///
/// 音声認識は語の一部を取り違えるため、完全一致では取り逃す。許容する誤りは語長に
/// 比例させ、短い語で誤爆しないようにする。
fn contains_close(text: &[char], form: &str) -> bool {
    let pattern: Vec<char> = form.chars().collect();
    if pattern.is_empty() {
        return false;
    }

    min_distance_to_substring(text, &pattern) <= pattern.len() / 4
}

/// パターンと、テキストの部分文字列との最小編集距離。
///
/// 先頭行を 0 で埋めることで、テキストのどの位置から照合を始めてもよいことを表す。
fn min_distance_to_substring(text: &[char], pattern: &[char]) -> usize {
    let mut prev = vec![0usize; text.len() + 1];
    let mut cur = vec![0usize; text.len() + 1];

    for (i, &p) in pattern.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &t) in text.iter().enumerate() {
            let substitute = prev[j] + usize::from(p != t);
            cur[j + 1] = substitute.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }

    prev.iter().copied().min().unwrap_or(pattern.len())
}

/// 照合用に読みを揃える。
///
/// 音声認識の結果は区切り記号や空白が入りうるため落とし、ひらがなはカタカナへ寄せる。
/// 長音符は直前の母音へ開く。同じ読みが「ターン」とも「たあん」とも書かれるため。
fn normalize(text: &str) -> String {
    let kana = text
        .chars()
        .filter(|c| !c.is_whitespace() && !is_separator(*c))
        .map(to_katakana);

    let mut normalized = String::new();
    for c in kana {
        if c == 'ー' {
            // 直前に母音がなければ開きようがないので落とす
            if let Some(vowel) = normalized.chars().last().and_then(vowel_of) {
                normalized.push(vowel);
            }
            continue;
        }
        normalized.push(c);
    }
    normalized
}

/// カタカナの母音。母音を持たない文字（「ン」「ッ」や漢字）は `None`。
fn vowel_of(c: char) -> Option<char> {
    const ROWS: [(char, &str); 5] = [
        ('ア', "アァカガサザタダナハバパマヤャラワヮ"),
        ('イ', "イィキギシジチヂニヒビピミリヰ"),
        ('ウ', "ウゥクグスズツヅヌフブプムユュルヴ"),
        ('エ', "エェケゲセゼテデネヘベペメレヱ"),
        ('オ', "オォコゴソゾトドノホボポモヨョロヲ"),
    ];

    ROWS.iter()
        .find(|(_, row)| row.contains(c))
        .map(|(vowel, _)| *vowel)
}

fn is_separator(c: char) -> bool {
    matches!(
        c,
        '、' | '。' | '，' | '．' | ',' | '.' | '!' | '?' | '！' | '？' | '・' | '-'
    )
}

/// ひらがなをカタカナへ寄せる。カタカナ・漢字・その他はそのまま。
fn to_katakana(c: char) -> char {
    match c {
        'ぁ'..='ゖ' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
        _ => c,
    }
}

/// トリガーワード検出に応じて行うターン境界の操作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TurnBoundary {
    /// ターンを開く
    Open,
    /// 進行中のターンを閉じる
    Close,
    /// 進行中のターンを閉じ、同じ位置で次のターンを開く
    Restart,
}

/// 検出をターン境界の操作へ変換する。
///
/// ターンは連続するため、進行中に開始ワードを検知したらそこで区切って次を開く。
/// 終了ワードは任意であり、開始ワードだけでも区間は途切れず確定する。
pub(super) struct BoundaryTracker {
    /// 直前の検出からこのサンプル数の間は次の検出を無視する。
    cooldown_samples: usize,
    last_detection: Option<usize>,
}

impl BoundaryTracker {
    pub(super) fn new(cooldown_samples: usize) -> Self {
        Self {
            cooldown_samples,
            last_detection: None,
        }
    }

    /// 検出を受け取り、行うべき境界操作を返す。`at` は検出位置の絶対インデックス。
    pub(super) fn accept(
        &mut self,
        hotword: Hotword,
        at: usize,
        turn_open: bool,
    ) -> Option<TurnBoundary> {
        if self.in_cooldown(at) {
            return None;
        }

        let boundary = match (hotword, turn_open) {
            (Hotword::TurnStart, false) => TurnBoundary::Open,
            (Hotword::TurnStart, true) => TurnBoundary::Restart,
            (Hotword::TurnEnd, true) => TurnBoundary::Close,
            // 開いていないターンは閉じられない
            (Hotword::TurnEnd, false) => return None,
        };

        self.last_detection = Some(at);
        Some(boundary)
    }

    /// 直前の検出から間もないか。同じ発話を重ねて拾うのを防ぐ。
    fn in_cooldown(&self, at: usize) -> bool {
        self.last_detection
            .is_some_and(|last| at.saturating_sub(last) < self.cooldown_samples)
    }
}

/// 既定のトリガーワード。
///
/// 漢字とかなの違いは照合で吸収できないため、想定する表記を並べる。
pub(super) const DEFAULT_START_WORDS: &[&str] = &["ターン開始", "ターンかいし"];
pub(super) const DEFAULT_END_WORDS: &[&str] = &["ターン終了", "ターンしゅうりょう"];

/// 認識プロンプトに載せる代表表記。
///
/// 照合用の表記ゆれは載せない。プロンプトはモデルに出させたい表記を示すものであり、
/// 揺れを並べると的が分散する。
pub(super) const PROMPT_WORDS: &[&str] = &[DEFAULT_START_WORDS[0], DEFAULT_END_WORDS[0]];

/// 同じ発話を重ねて拾わないためのクールダウン。
pub(super) const COOLDOWN_SECS: f32 = 3.0;

#[cfg(test)]
mod tests {
    use super::*;

    fn matcher() -> HotwordMatcher {
        HotwordMatcher::new(&["ターン開始"], &["ターン終了"])
    }

    #[test]
    fn absorbs_a_long_vowel_written_as_a_kana() {
        let matcher = HotwordMatcher::new(&["ターンかいし"], &["ターンしゅうりょう"]);

        // 「ター」と「たあ」は同じ読み
        assert_eq!(matcher.find("たあんかいし"), Some(Hotword::TurnStart));
    }

    #[test]
    fn tolerates_a_single_misheard_character() {
        let matcher = HotwordMatcher::new(&["ターンかいし"], &["ターンしゅうりょう"]);

        // 実際の認識結果。「ターン」が「はあん」に化けている
        assert_eq!(matcher.find("はあん、かいし"), Some(Hotword::TurnStart));
    }

    #[test]
    fn does_not_match_unrelated_speech_of_a_similar_length() {
        let matcher = HotwordMatcher::new(&["ターンかいし"], &["ターンしゅうりょう"]);

        assert_eq!(matcher.find("さっきのタスクやった"), None);
        assert_eq!(matcher.find("カフェテリアにいた"), None);
    }

    #[test]
    fn finds_the_start_word_in_surrounding_text() {
        assert_eq!(
            matcher().find("じゃあターン開始します"),
            Some(Hotword::TurnStart)
        );
    }

    #[test]
    fn absorbs_kana_and_separator_differences() {
        let matcher = HotwordMatcher::new(&["ターンかいし"], &["ターンしゅうりょう"]);

        // ひらがな・カタカナの違いを無視する
        assert_eq!(matcher.find("ターンカイシ"), Some(Hotword::TurnStart));
        // 区切り記号や空白が混ざっても拾う
        assert_eq!(
            matcher.find("ターン、しゅう りょう"),
            Some(Hotword::TurnEnd)
        );
    }

    #[test]
    fn returns_none_without_a_trigger_word() {
        assert_eq!(matcher().find("エレキで死体見つけた"), None);
    }

    /// クールダウンは検証したい振る舞いではないので、十分短くしておく。
    fn tracker() -> BoundaryTracker {
        BoundaryTracker::new(0)
    }

    #[test]
    fn start_word_opens_a_turn() {
        assert_eq!(
            tracker().accept(Hotword::TurnStart, 100, false),
            Some(TurnBoundary::Open)
        );
    }

    #[test]
    fn start_word_during_a_turn_restarts_it() {
        assert_eq!(
            tracker().accept(Hotword::TurnStart, 100, true),
            Some(TurnBoundary::Restart)
        );
    }

    #[test]
    fn end_word_closes_an_open_turn() {
        assert_eq!(
            tracker().accept(Hotword::TurnEnd, 100, true),
            Some(TurnBoundary::Close)
        );
    }

    #[test]
    fn end_word_without_an_open_turn_is_ignored() {
        assert_eq!(tracker().accept(Hotword::TurnEnd, 100, false), None);
    }

    #[test]
    fn detections_during_cooldown_are_ignored() {
        let mut tracker = BoundaryTracker::new(50);

        assert_eq!(
            tracker.accept(Hotword::TurnStart, 100, false),
            Some(TurnBoundary::Open)
        );
        // 同じ発話を重ねて拾わない
        assert_eq!(tracker.accept(Hotword::TurnStart, 140, true), None);
        // クールダウンを過ぎれば拾う
        assert_eq!(
            tracker.accept(Hotword::TurnStart, 150, true),
            Some(TurnBoundary::Restart)
        );
    }

    #[test]
    fn ignored_detections_do_not_extend_the_cooldown() {
        let mut tracker = BoundaryTracker::new(50);
        tracker.accept(Hotword::TurnStart, 100, false);

        // 無視された検出でクールダウンの起点がずれると、次が拾えなくなる
        tracker.accept(Hotword::TurnStart, 140, true);

        assert_eq!(
            tracker.accept(Hotword::TurnStart, 150, true),
            Some(TurnBoundary::Restart)
        );
    }
}
