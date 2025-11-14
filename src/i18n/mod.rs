pub mod ja;
pub mod en;
pub mod keys;
pub mod words;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Japanese,
    English,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::Japanese => "ja",
            Language::English => "en",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Language::Japanese => "日本語",
            Language::English => "English",
        }
    }

    pub fn all() -> Vec<Language> {
        vec![Language::Japanese, Language::English]
    }
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

    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
    }

    pub fn current_language(&self) -> Language {
        self.current_language
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

    // 動的な値を含む翻訳用のヘルパー
    pub fn t_with_number(&self, key: &str, number: usize) -> String {
        format!("{} {}", self.t(key), number)
    }

    pub fn t_with_params(&self, key: &str, params: &[&str]) -> String {
        let mut result = self.t(key).to_string();
        for (i, param) in params.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), param);
        }
        result
    }
}