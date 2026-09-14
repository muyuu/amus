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
