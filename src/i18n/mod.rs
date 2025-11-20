pub mod en;
pub mod ja;
pub mod keys;
pub mod words;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Japanese,
    English,
}

impl Default for Language {
    fn default() -> Self {
        Language::Japanese
    }
}

pub struct Translator {
    current_language: Language,
    translations: HashMap<Language, HashMap<&'static str, String>>,
}

impl Translator {
    pub fn new(language: Language) -> Self {
        let mut translations = HashMap::new();

        // 各言語の翻訳辞書を読み込み
        translations.insert(Language::Japanese, ja::get_translations());
        translations.insert(Language::English, en::get_translations());

        Self {
            current_language: language,
            translations,
        }
    }

    pub fn t<'a>(&'a self, key: &str) -> &'a str {
        if let Some(dict) = self.translations.get(&self.current_language) {
            if let Some(translation) = dict.get(key) {
                return translation;
            }
        }

        // フォールバック: 日本語で試す
        if self.current_language != Language::Japanese {
            if let Some(dict) = self.translations.get(&Language::Japanese) {
                if let Some(translation) = dict.get(key) {
                    return translation;
                }
            }
        }

        // 最終フォールバック: キーをそのまま返す（これは問題があるので修正が必要）
        "Missing translation"
    }
}
