use crate::Error;
use crate::parse::{DictPage, example, inner_text};
use scraper::Selector;
use serde::{Deserialize, Serialize};

pub trait JapaneseParser {
    fn to_chinese(&self) -> std::result::Result<ToChinese, Error>;
    fn to_japanese(&self) -> std::result::Result<ToJapanese, Error>;
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Example {
    pub ja: String,
    pub zh: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Pronunciation {
    pub kana: String,
    pub romaji: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToChinese {
    pub input_text: String,
    pub pronunciation: Option<Pronunciation>,
    pub meanings: Vec<String>,
    pub part_of_speech: Option<String>,
    pub exam: Option<String>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToJapanese {
    pub input_text: String,
    pub meanings: Vec<Meaning>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Meaning {
    pub point: String,
    pub definition: String,
}

#[rustfmt::skip]
mod selectors {
    use std::sync::LazyLock;
    use crate::selector;
    use super::Selector;

    pub mod jc {
        use super::{selector, LazyLock, Selector};

        pub static WORD:            LazyLock<Selector> = selector!("div.word-head div.title");
        pub static KANA:            LazyLock<Selector> = selector!("div.head-content > span:not(.divider):not(.tone)");
        pub static ROMAJI:          LazyLock<Selector> = selector!("div.head-content > span.tone");
        pub static SENSE:           LazyLock<Selector> = selector!("div.each-sense");
        pub static SENSE_TEXT:      LazyLock<Selector> = selector!("div.sense-con div.sense-ja");
        pub static PART_OF_SPEECH:  LazyLock<Selector> = selector!("div.pos-line div.word-pos");
        pub static EXAM:            LazyLock<Selector> = selector!("div.label");
    }

    pub mod cj {
        use super::{selector, LazyLock, Selector};

        pub static WORD:            LazyLock<Selector> = selector!("div.word-head div.title");
        pub static MEANINGS:        LazyLock<Selector> = selector!("div.cj_data_list");
        pub static POINT:           LazyLock<Selector> = selector!("a.point");
        pub static JMSY:            LazyLock<Selector> = selector!("div.cj_jmsy");
    }

}

impl JapaneseParser for DictPage<'_> {
    fn to_chinese(&self) -> std::result::Result<ToChinese, Error> {
        let input_text = self.child_text(&selectors::jc::WORD);

        let kana = self
            .0
            .select(&selectors::jc::KANA)
            .next()
            .map(|el| {
                inner_text(&el)
                    .chars()
                    .filter(|c| {
                        !matches!(
                            c,
                            '①' | '②'
                                | '③'
                                | '④'
                                | '⑤'
                                | '⑥'
                                | '⑦'
                                | '⑧'
                                | '⑨'
                                | '⑩'
                                | '⓪'
                        )
                    })
                    .collect::<String>()
            })
            .filter(|s| !s.is_empty());

        let romaji = self
            .0
            .select(&selectors::jc::ROMAJI)
            .next()
            .map(|el| inner_text(&el))
            .filter(|s| !s.is_empty());

        let pronunciation = kana
            .zip(romaji)
            .map(|(kana, romaji)| Pronunciation { kana, romaji });

        let part_of_speech = self
            .0
            .select(&selectors::jc::PART_OF_SPEECH)
            .next()
            .map(|el| {
                inner_text(&el)
                    .trim_matches('[')
                    .trim_matches(']')
                    .to_owned()
            })
            .filter(|s| !s.is_empty());

        let exam = self
            .0
            .select(&selectors::jc::EXAM)
            .next()
            .map(|el| inner_text(&el))
            .filter(|s| !s.is_empty());

        let meanings: Vec<String> = self
            .0
            .select(&selectors::jc::SENSE)
            .flat_map(|sense| sense.select(&selectors::jc::SENSE_TEXT).collect::<Vec<_>>())
            .map(|ja| inner_text(&ja))
            .filter(|text| !text.is_empty() && text != &input_text)
            .collect();

        if meanings.is_empty() {
            return Err(Error::NoTranslationResults);
        }

        let examples = example::extract_examples(&self.0)
            .into_iter()
            .map(|pair| Example {
                ja: pair.source,
                zh: pair.target,
            })
            .collect();

        Ok(ToChinese {
            input_text,
            pronunciation,
            meanings,
            part_of_speech,
            exam,
            examples,
        })
    }

    fn to_japanese(&self) -> std::result::Result<ToJapanese, Error> {
        let input_text = self.child_text(&selectors::cj::WORD);

        let mut meanings = Vec::new();
        for element in self.0.select(&selectors::cj::MEANINGS) {
            let point = element
                .select(&selectors::cj::POINT)
                .next()
                .map(|el| inner_text(&el))
                .unwrap_or_default();

            let definition = element
                .select(&selectors::cj::JMSY)
                .next()
                .map(|el| inner_text(&el))
                .unwrap_or_default();

            if !definition.is_empty() {
                meanings.push(Meaning { point, definition });
            }
        }

        if meanings.is_empty() {
            return Err(Error::NoTranslationResults);
        }

        let examples = example::extract_examples(&self.0)
            .into_iter()
            .map(|pair| Example {
                ja: pair.target,
                zh: pair.source,
            })
            .collect();

        Ok(ToJapanese {
            input_text,
            meanings,
            examples,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_from_japanese() {
        let doc = Html::parse_document(include_str!("../fixtures/where_jc.html"));
        let dp = DictPage::new(doc.select(&crate::parse::selectors::BODY).next().unwrap());
        assert_eq!(
            <DictPage as JapaneseParser>::to_chinese(&dp).unwrap(),
            ToChinese {
                input_text: "どこ".into(),
                pronunciation: Some(Pronunciation {
                    kana: "どこ".into(),
                    romaji: "doko".into()
                }),
                part_of_speech: Some("代词".into()),
                exam: None,
                meanings: vec![
                    "何处，哪里，哪儿。".into(),
                    "どこからともなく現れる。也不知从哪儿出现了。".into(),
                    "哪儿（どういう点）；怎么。".into(),
                ],
                examples: vec![
                    Example {
                        ja: "どことどこの国へ行ったことがあるの？".into(),
                        zh: "你去过哪里和哪个国家?".into(),
                    },
                    Example {
                        ja: "どこからどこまでが私の責任ですか。".into(),
                        zh: "从哪里到哪里是我的责任?".into(),
                    },
                    Example {
                        ja: "コンビニはどこですか。".into(),
                        zh: "便利店在哪里?".into(),
                    },
                ],
            }
        );
    }

    #[test]
    fn test_to_japanese() {
        let doc = Html::parse_document(include_str!("../fixtures/where_cj.html"));
        let dp = DictPage::new(doc.select(&crate::parse::selectors::BODY).next().unwrap());
        assert_eq!(
            <DictPage as JapaneseParser>::to_japanese(&dp).unwrap(),
            ToJapanese {
                input_text: "哪里".into(),
                meanings: vec![
                    Meaning { point: "か".into(), definition: "什么时候；几；多少；谁；哪里；什么。接在疑问词或表示疑问的词后表示不确定。".into() },
                    Meaning { point: "何".into(), definition: "哪里，没什么。".into() },
                    Meaning { point: "どこ".into(), definition: "何处，哪里，哪儿。".into() },
                    Meaning { point: "何が".into(), definition: "何事，哪里，怎么。".into() },
                    Meaning { point: "そんな".into(), definition: "哪里，不会。".into() },
                ],
                examples: vec![
                    Example { ja: "どこからどこまでが私の責任ですか。".into(), zh: "从哪里到哪里是我的责任?".into() },
                    Example { ja: "ありがとうございます。—いやいや。".into(), zh: "谢谢您。—哪里哪里。".into() },
                    Example { ja: "こちらこそよろしくお願いします。".into(), zh: "哪里哪里请多多关照。".into() },
                ],
            }
        );
    }
}
