pub mod en;
pub mod ja;
pub mod keys;
pub mod words;

use keys::TextKey;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    #[default]
    Japanese,
    // 対応言語。言語切り替え UI から選択されるまでコード上は構築されない。
    #[allow(dead_code)]
    English,
}

pub struct Translator {
    current_language: Language,
    /// 言語ごとの翻訳テーブル。`TextKey as usize` でインデックスする。
    /// 起動時に一度だけ構築し、`t` は配列参照だけで済ませる（毎フレームの
    /// ハッシュ・確保を避ける）。
    japanese: Vec<String>,
    english: Vec<String>,
}

impl Translator {
    pub fn new(language: Language) -> Self {
        Self {
            current_language: language,
            japanese: build_table(ja::translate),
            english: build_table(en::translate),
        }
    }

    pub fn t(&self, key: TextKey) -> &str {
        &self.table()[key as usize]
    }

    fn table(&self) -> &[String] {
        match self.current_language {
            Language::Japanese => &self.japanese,
            Language::English => &self.english,
        }
    }
}

/// 全キーを宣言順に変換し、`TextKey as usize` で引ける配列を作る。
fn build_table(translate: fn(TextKey) -> String) -> Vec<String> {
    TextKey::ALL.iter().map(|&key| translate(key)).collect()
}

#[cfg(test)]
mod tests {
    use super::keys::TextKey;
    use super::{Language, Translator};

    #[test]
    fn t_resolves_per_language() {
        let ja = Translator::new(Language::Japanese);
        let en = Translator::new(Language::English);
        // 同じキーが言語ごとに別の文字列へ解決される
        assert_eq!(ja.t(TextKey::SetupCancel), "キャンセル");
        assert_eq!(en.t(TextKey::SetupCancel), "Cancel");
    }

    #[test]
    fn table_covers_every_key() {
        // ビルド済みテーブルが全キー分の要素を持つ（index 漏れがない）
        let ja = Translator::new(Language::Japanese);
        for &key in TextKey::ALL {
            assert!(!ja.t(key).is_empty(), "未翻訳のキーがある: {:?}", key);
        }
    }
}
