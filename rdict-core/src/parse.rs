use anyhow::{Result, ensure};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Pronunciation {
    pub uk: Option<String>,
    pub us: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Example {
    pub en: String,
    pub zh: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Meaning {
    pub part_of_speech: Option<String>,
    pub definitions: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ToChinese {
    pub input_text: String,
    pub pronunciation: Pronunciation,
    pub meanings: Vec<Meaning>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ToEnglish {
    pub input_text: String,
    pub meanings: Vec<String>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TranslationData {
    #[serde(rename = "to_chinese")]
    ToChinese(ToChinese),

    #[serde(rename = "to_english")]
    ToEnglish(ToEnglish),
}

/// Parses English, returns Chinese
pub fn to_chinese(input_text: &str, html: &str) -> Result<ToChinese> {
    let document = Html::parse_document(html);
    let mut result = ToChinese::default();
    result.input_text = input_text.to_string();

    // Pronunciation
    let pronunciation_selector = Selector::parse(".phone_con .per-phone .phonetic").unwrap();
    for (i, element) in document.select(&pronunciation_selector).take(2).enumerate() {
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

    // Translations
    let meanings_selector = Selector::parse(".trans-container .basic .word-exp").unwrap();
    let definitions_selector = Selector::parse(".trans").unwrap();
    let part_of_speech_selector = Selector::parse(".pos").unwrap();

    for element in document.select(&meanings_selector) {
        let part_of_speech = element
            .select(&part_of_speech_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_owned());

        let definitions: Vec<String> = element
            .select(&definitions_selector)
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

    // Example sentences
    let example_selector = Selector::parse(".trans-container .mcols-layout .col2").unwrap();
    let en_selector = Selector::parse(".sen-eng").unwrap();
    let zh_selector = Selector::parse(".sen-ch").unwrap();

    for element in document.select(&example_selector) {
        let en = element
            .select(&en_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        let zh = element
            .select(&zh_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        if !en.is_empty() || !zh.is_empty() {
            result.examples.push(Example { en, zh });
        }
    }

    ensure!(
        !result.examples.is_empty()
            || !result.meanings.is_empty()
            || result.pronunciation.uk.is_some()
            || result.pronunciation.us.is_some(),
        "No translation results found for English to Chinese"
    );

    Ok(result)
}

/// Parses Chinese, returns English
pub fn to_english(input_text: &str, html: &str) -> Result<ToEnglish> {
    let document = Html::parse_document(html);
    let mut result = ToEnglish::default();
    result.input_text = input_text.to_string();

    // Meanings
    let translation_selector = Selector::parse(".trans-container .basic .col2 .point").unwrap();
    for element in document.select(&translation_selector) {
        let text = element.text().collect::<String>().trim().to_owned();
        if !text.is_empty() {
            result.meanings.push(text);
        }
    }

    // Example sentences
    let example_selector = Selector::parse(".trans-container .mcols-layout .col2").unwrap();
    let en_selector = Selector::parse(".sen-eng").unwrap();
    let zh_selector = Selector::parse(".sen-ch").unwrap();

    for element in document.select(&example_selector) {
        let en = element
            .select(&en_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        let zh = element
            .select(&zh_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        if !en.is_empty() || !zh.is_empty() {
            result.examples.push(Example { en, zh });
        }
    }

    ensure!(
        !result.examples.is_empty() || !result.meanings.is_empty(),
        "No translation results found for Chinese To English"
    );

    Ok(result)
}
