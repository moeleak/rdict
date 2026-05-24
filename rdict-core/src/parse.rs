use crate::Error;
use scraper::{ElementRef, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Pronunciation {
    pub uk: Option<String>,
    pub us: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Example {
    pub en: String,
    pub zh: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Meaning {
    pub part_of_speech: Option<String>,
    pub definitions: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToChinese {
    pub input_text: String,
    pub pronunciation: Pronunciation,
    pub meanings: Vec<Meaning>,
    pub examples: Vec<Example>,
    pub exams: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToEnglish {
    pub input_text: String,
    pub meanings: Vec<String>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NotFound {
    pub suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum TranslationData {
    #[serde(rename = "to_chinese")]
    ToChinese(ToChinese),

    #[serde(rename = "to_english")]
    ToEnglish(ToEnglish),

    #[serde(rename = "not_found")]
    NotFound(NotFound),
}

macro_rules! selector {
    ($css:expr) => {
        // Selectors are parsed at runtime...
        LazyLock::new(|| Selector::parse($css).expect(concat!("Invalid selector: ", $css)))
    };
}

#[rustfmt::skip]
pub mod selectors {
    use std::sync::LazyLock;

    use super::Selector;

    pub static BODY_SELECTOR:                   LazyLock<Selector> = selector!(".search_result-dict");
    pub static WORD_SELECTOR:                   LazyLock<Selector> = selector!(".word-head .title");
    pub static EXAM_LIST_SELECTOR:              LazyLock<Selector> = selector!(".exam_type"); 
    pub static EXAM_SELECTOR:                   LazyLock<Selector> = selector!(".exam_type-value");
    pub static PRONUNCIATION_SELECTOR:          LazyLock<Selector> = selector!(".phone_con .per-phone .phonetic");
    pub static MEANINGS_SELECTOR:               LazyLock<Selector> = selector!(".trans-container .basic .word-exp");
    pub static DEFINITIONS_SELECTOR:            LazyLock<Selector> = selector!(".trans");
    pub static PART_OF_SPEECH_SELECTOR:         LazyLock<Selector> = selector!(".pos");
    pub static EXAMPLE_SELECTOR:                LazyLock<Selector> = selector!(".trans-container .mcols-layout .col2");
    pub static EN_SELECTOR:                     LazyLock<Selector> = selector!(".sen-eng");
    pub static ZH_SELECTOR:                     LazyLock<Selector> = selector!(".sen-ch");
    pub static TO_ENGLISH_TRANSLATION_SELECTOR: LazyLock<Selector> = selector!(".trans-container .basic .col2 .point");
}

/// Parses English, returns Chinese
pub fn to_chinese(input_text: &str, document: ElementRef) -> std::result::Result<ToChinese, Error> {
    let mut result = ToChinese {
        input_text: input_text.to_owned(),
        ..Default::default()
    };

    for (i, element) in document
        .select(&selectors::PRONUNCIATION_SELECTOR)
        .take(2)
        .enumerate()
    {
        let text = element
            .text()
            .collect::<String>()
            .trim_matches('/')
            .trim()
            .to_owned();

        let text = if text.is_empty() { None } else { Some(text) };

        match i {
            0 => result.pronunciation.uk = text,
            1 => result.pronunciation.us = text,
            _ => unreachable!(),
        }
    }

    for element in document.select(&selectors::MEANINGS_SELECTOR) {
        let part_of_speech = element
            .select(&selectors::PART_OF_SPEECH_SELECTOR)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_owned());

        let definitions: Vec<String> = element
            .select(&selectors::DEFINITIONS_SELECTOR)
            .next()
            .map(|e| {
                e.text()
                    .collect::<String>()
                    .trim()
                    .split('；')
                    .map(|s| s.trim().to_owned())
                    .collect()
            })
            .unwrap_or_default();

        if part_of_speech.is_some() || !definitions.is_empty() {
            result.meanings.push(Meaning {
                part_of_speech,
                definitions,
            });
        }
    }

    // Exams
    // e.g. [ "初中", "高中", "CET4", "CET6", "考研" ]
    if let Some(container) = document.select(&selectors::EXAM_LIST_SELECTOR).next() {
        for exam_elem in container.select(&selectors::EXAM_SELECTOR) {
            let exam_text = exam_elem.text().collect::<String>().trim().to_owned();

            if !exam_text.is_empty() {
                result.exams.push(exam_text);
            }
        }
    }

    // Example sentences
    for element in document.select(&selectors::EXAMPLE_SELECTOR) {
        let en = element
            .select(&selectors::EN_SELECTOR)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        let zh = element
            .select(&selectors::ZH_SELECTOR)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        if !en.is_empty() || !zh.is_empty() {
            result.examples.push(Example { en, zh });
        }
    }

    if result.examples.is_empty()
        && result.meanings.is_empty()
        && result.pronunciation.uk.is_none()
        && result.pronunciation.us.is_none()
    {
        return Err(Error::NoTranslationResults);
    }

    Ok(result)
}

/// Parses Chinese, returns English
pub fn to_english(input_text: &str, document: ElementRef) -> std::result::Result<ToEnglish, Error> {
    let mut result = ToEnglish {
        input_text: input_text.to_owned(),
        ..Default::default()
    };

    // Meanings
    for element in document.select(&selectors::TO_ENGLISH_TRANSLATION_SELECTOR) {
        let text = element.text().collect::<String>().trim().to_owned();
        if !text.is_empty() {
            result.meanings.push(text);
        }
    }

    // Example sentences
    for element in document.select(&selectors::EXAMPLE_SELECTOR) {
        let en = element
            .select(&selectors::EN_SELECTOR)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        let zh = element
            .select(&selectors::ZH_SELECTOR)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        if !en.is_empty() || !zh.is_empty() {
            result.examples.push(Example { en, zh });
        }
    }

    if result.examples.is_empty() && result.meanings.is_empty() {
        return Err(Error::NoTranslationResults);
    }

    Ok(result)
}

pub fn not_found(document: ElementRef) -> std::result::Result<NotFound, Error> {
    let maybe_container_selector = Selector::parse("div.maybe").unwrap();
    let word_selector = Selector::parse("div.maybe_word a.point").unwrap();

    let mut suggestions = Vec::new();

    // Assuming `document` is your parsed Html object and `result` is your NotFound struct
    if let Some(container) = document.select(&maybe_container_selector).next() {
        // Loop through every <a class="point"> found inside the container
        for anchor in container.select(&word_selector) {
            let suggestion_text = anchor.text().collect::<String>().trim().to_owned();

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
