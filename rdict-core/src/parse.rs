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

#[rustfmt::skip]
pub mod selectors {
    use super::Selector;
    use std::sync::LazyLock;

    pub static BODY:            LazyLock<Selector> = selector!("div.search_result-dict");
    pub static MAYBE:           LazyLock<Selector> = selector!("div.maybe");
    pub static MAYBE_WORD:      LazyLock<Selector> = selector!("div.maybe_word a.point");
    pub static DIRECTION_JC:    LazyLock<Selector> = selector!("div.page.newjc");
    pub static DIRECTION:       LazyLock<Selector> = selector!("div.dict-module");
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
        let mut suggestions = Vec::new();

        if let Some(container) = self.0.select(&selectors::MAYBE).next() {
            for anchor in container.select(&selectors::MAYBE_WORD) {
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

    pub fn parse_translation_direction(&self) -> Result<TranslationData, Error> {
        // ja -> zh uses a distinct page layout class
        if self.0.select(&selectors::DIRECTION_JC).next().is_some() {
            return Ok(TranslationData::FromJapanese(
                parse::ja::JapaneseParser::to_chinese(self)?,
            ));
        }

        let target_direction = self.0.select(&selectors::DIRECTION).find_map(|el| {
            let directions = ["ec", "ce", "fc", "cf", "kc", "ck", "cj"];

            directions.into_iter().find(|&dir| {
                el.value()
                    .has_class(dir, scraper::CaseSensitivity::AsciiCaseInsensitive)
            })
        });

        match target_direction {
            Some("ec") => Ok(TranslationData::FromEnglish(
                parse::en::EnglishParser::to_chinese(self)?,
            )),
            Some("ce") => Ok(TranslationData::ToEnglish(
                parse::en::EnglishParser::to_english(self)?,
            )),
            Some("fc") => Ok(TranslationData::FromFrench(
                parse::fr::FrenchParser::to_chinese(self)?,
            )),
            Some("cf") => Ok(TranslationData::ToFrench(
                parse::fr::FrenchParser::to_french(self)?,
            )),
            Some("kc") => Ok(TranslationData::FromKorean(
                parse::ko::KoreanParser::to_chinese(self)?,
            )),
            Some("ck") => Ok(TranslationData::ToKorean(
                parse::ko::KoreanParser::to_korean(self)?,
            )),
            Some("cj") => Ok(TranslationData::ToJapanese(
                parse::ja::JapaneseParser::to_japanese(self)?,
            )),
            _ => Err(Error::NoTranslationResults),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::en::EnglishParser;
    use scraper::Html;

    #[test]
    fn test_parse_translation_direction() {
        let doc = Html::parse_document(include_str!("./fixtures/where_ec.html"));
        let dp = DictPage::new(doc.select(&crate::parse::selectors::BODY).next().unwrap());

        assert_eq!(
            dp.parse_translation_direction().unwrap(),
            TranslationData::FromEnglish(<DictPage as EnglishParser>::to_chinese(&dp).unwrap())
        )
    }
}
