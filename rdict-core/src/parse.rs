pub mod en;
pub mod example;
pub mod fr;
pub mod ja;
pub mod ko;

use crate::{Error, parse, rdict::TranslationData};
use scraper::{ElementRef, Node, Selector};
use serde::{Deserialize, Serialize};

#[must_use]
pub fn inner_text(el: &ElementRef) -> String {
    el.text().collect::<String>().trim().to_owned()
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NotFound {
    pub suggestions: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExamplePair {
    pub source: String,
    pub target: String,
}

#[macro_export]
macro_rules! selector {
    ($css:expr) => {
        // Selectors are parsed at runtime...
        LazyLock::new(|| Selector::parse($css).expect(concat!("Invalid selector: ", $css)))
    };
}

pub mod selectors {
    use super::Selector;
    use std::sync::LazyLock;

    pub static BODY: LazyLock<Selector> = selector!("div.search_result-dict");
}

pub struct DictPage<'a>(ElementRef<'a>);

impl<'a> DictPage<'a> {
    #[must_use]
    pub const fn new(element: ElementRef<'a>) -> Self {
        Self(element)
    }

    #[must_use]
    pub fn child_text(&self, selector: &Selector) -> String {
        self.0
            .select(selector)
            .next()
            .map(|el| {
                el.children()
                    .filter_map(|child| {
                        if let Node::Text(text_node) = child.value() {
                            Some::<&str>(text_node.as_ref())
                        } else {
                            None
                        }
                    })
                    .collect::<String>()
                    .trim()
                    .to_owned()
            })
            .unwrap_or_default()
    }

    pub fn not_found(&self) -> std::result::Result<NotFound, Error> {
        let maybe_container_selector = Selector::parse("div.maybe").unwrap();
        let word_selector = Selector::parse("div.maybe_word a.point").unwrap();

        let mut suggestions = Vec::new();

        if let Some(container) = self.0.select(&maybe_container_selector).next() {
            for anchor in container.select(&word_selector) {
                let suggestion_text = inner_text(&anchor);

                if !suggestion_text.is_empty() {
                    suggestions.push(suggestion_text);
                }
            }
        }

        if !suggestions.is_empty() {
            return Ok(NotFound { suggestions });
        }

        Err(Error::NoTranslationResults)
    }

    /// Check if any descendant of search_result-dict is a div with the given dict-module class.
    fn has_module(&self, class: &str) -> bool {
        Selector::parse(&format!("div.{class}.dict-module"))
            .is_ok_and(|sel| self.0.select(&sel).next().is_some())
    }

    /// Check if a `div.page.newXX` (used by Japanese) exists.
    fn has_page_mod(&self, class: &str) -> bool {
        Selector::parse(&format!("div.page.{class}"))
            .is_ok_and(|sel| self.0.select(&sel).next().is_some())
    }

    /// Detect which dict-module is present and call the right parser.
    pub fn parse_translation(&self) -> Result<crate::rdict::TranslationData, Error> {
        let modules = ["ec", "ce", "fc", "cf", "ck", "kc", "cj"];
        let page_mods = ["newjc"];

        let active_module = modules.iter().find(|&&m| self.has_module(m));
        let active_page = page_mods.iter().find(|&&m| self.has_page_mod(m));

        match (active_module, active_page) {
            (Some(&"ec"), _) => Ok(TranslationData::FromEnglish(
                parse::en::EnglishParser::to_chinese(self)?,
            )),
            (Some(&"ce"), _) => Ok(TranslationData::ToEnglish(
                parse::en::EnglishParser::to_english(self)?,
            )),

            (Some(&"fc"), _) => Ok(TranslationData::FromFrench(
                parse::fr::FrenchParser::to_chinese(self)?,
            )),
            (Some(&"cf"), _) => Ok(TranslationData::ToFrench(
                parse::fr::FrenchParser::to_french(self)?,
            )),

            (Some(&"kc"), _) => Ok(TranslationData::FromKorean(
                parse::ko::KoreanParser::to_chinese(self)?,
            )),
            (Some(&"ck"), _) => Ok(TranslationData::ToKorean(
                parse::ko::KoreanParser::to_korean(self)?,
            )),

            (Some(&"cj"), _) => Ok(TranslationData::ToJapanese(
                parse::ja::JapaneseParser::to_japanese(self)?,
            )),

            (_, Some(&"newjc")) => Ok(TranslationData::FromJapanese(
                parse::ja::JapaneseParser::to_chinese(self)?,
            )),

            _ => Err(Error::NoTranslationResults),
        }
    }
}
