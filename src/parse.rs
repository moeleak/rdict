use scraper::{Html, Selector};

#[derive(Debug, Default)]
pub struct Phonetic {
    pub uk: String,
    pub us: String,
}

#[derive(Debug, Default)]
pub struct ExampleSentense {
    pub english_sentense: String,
    pub chinese_sentense: String,
}

#[derive(Debug, Default)]
pub struct ToChineseTranslation {
    pub english_word_type: String,
    pub chinese_translation: String,
}

#[derive(Debug, Default)]
pub struct ToChinese {
    pub phonetic: Phonetic,
    pub translations: Vec<ToChineseTranslation>,
    pub example_sentenses: Vec<ExampleSentense>,
}

#[derive(Debug, Default)]
pub struct ToEnglish {
    pub translations: Vec<String>,
    pub example_sentenses: Vec<ExampleSentense>,
}

pub fn to_chinese(html: &str) -> Result<ToChinese, String> {
    let document = Html::parse_document(html);
    let mut result = ToChinese::default();

    // Phonetic
    let per_phone_selector = Selector::parse(".phone_con .per-phone").unwrap();
    let phonetic_selector = Selector::parse(".phonetic").unwrap();

    for (i, element) in document.select(&per_phone_selector).enumerate() {
        let text = element
            .select(&phonetic_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if i == 0 {
            result.phonetic.uk = text;
        } else if i == 1 {
            result.phonetic.us = text;
        }
    }

    // Translations
    let word_exp_selector = Selector::parse(".trans-container .basic .word-exp").unwrap();
    let trans_selector = Selector::parse(".trans").unwrap();
    let pos_selector = Selector::parse(".pos").unwrap();

    for element in document.select(&word_exp_selector) {
        let chinese_translation = element
            .select(&trans_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let english_word_type = element
            .select(&pos_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if !chinese_translation.is_empty() || !english_word_type.is_empty() {
            result.translations.push(ToChineseTranslation {
                chinese_translation,
                english_word_type,
            });
        }
    }

    // Example sentenses
    let example_selector = Selector::parse(".trans-container .mcols-layout .col2").unwrap();
    let sen_eng_selector = Selector::parse(".sen-eng").unwrap();
    let sen_ch_selector = Selector::parse(".sen-ch").unwrap();

    for element in document.select(&example_selector) {
        let english_sentense = element
            .select(&sen_eng_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let chinese_sentense = element
            .select(&sen_ch_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();


        if !english_sentense.is_empty() || !chinese_sentense.is_empty() {
            result.example_sentenses.push(ExampleSentense {
                english_sentense,
                chinese_sentense
            });
        }
    }

    if result.translations.is_empty() {
        return Err("no translation results found for English to Chinese".to_string());
    }

    Ok(result)
}

pub fn to_english(html: &str) -> Result<ToEnglish, String> {
    let document = Html::parse_document(html);
    let mut result = ToEnglish::default();

    // Translations
    let translation_selector = Selector::parse(".trans-container .basic .col2 .point").unwrap();
    for element in document.select(&translation_selector) {
        let text = element.text().collect::<String>().trim().to_string();
        if !text.is_empty() {
            result.translations.push(text);
        }
    }

    // Example sentenses
    let example_selector = Selector::parse(".trans-container .mcols-layout .col2").unwrap();
    let sen_eng_selector = Selector::parse(".sen-eng").unwrap();
    let sen_ch_selector = Selector::parse(".sen-ch").unwrap();

    for element in document.select(&example_selector) {
        let english_sentense = element
            .select(&sen_eng_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let chinese_sentense = element
            .select(&sen_ch_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();


        if !english_sentense.is_empty() || !chinese_sentense.is_empty() {
            result.example_sentenses.push(ExampleSentense {
                english_sentense,
                chinese_sentense,
            });
        }
    }

    if result.translations.is_empty() {
        return Err("no translation results found for Chinese to English".to_string());
    }

    Ok(result)
}
