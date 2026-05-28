use crate::Error;
use crate::parse::{DictPage, example, inner_text};
use scraper::Selector;
use serde::{Deserialize, Serialize};

pub trait EnglishParser {
    fn to_chinese(&self) -> std::result::Result<ToChinese, Error>;
    fn to_english(&self) -> std::result::Result<ToEnglish, Error>;
}

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

#[rustfmt::skip]
mod selectors {
    use std::sync::LazyLock;
    use crate::selector;
    use super::Selector;

    pub mod common {
        use super::*;

        pub static WORD:            LazyLock<Selector> = selector!("div.word-head div.title");
    }

    pub mod ec {
        use super::*;

        pub static EXAM_LIST:       LazyLock<Selector> = selector!("div.exam_type"); 
        pub static EXAM:            LazyLock<Selector> = selector!("span.exam_type-value");
        pub static PRONUNCIATION:   LazyLock<Selector> = selector!("div.phone_con div.per-phone span.phonetic");
        pub static MEANINGS:        LazyLock<Selector> = selector!(".trans-container .basic .word-exp");
        pub static DEFINITIONS:     LazyLock<Selector> = selector!("span.trans");
        pub static PART_OF_SPEECH:  LazyLock<Selector> = selector!("span.pos");
    }

    pub mod ce {
        use super::*;

        pub static POINT:           LazyLock<Selector> = selector!(".trans-container .basic .col2 .point");
    }
}

impl EnglishParser for DictPage<'_> {
    fn to_chinese(&self) -> std::result::Result<ToChinese, Error> {
        let input_text = self.child_text(&selectors::common::WORD);

        let mut uk = None;
        let mut us = None;
        for (i, element) in self
            .0
            .select(&selectors::ec::PRONUNCIATION)
            .take(2)
            .enumerate()
        {
            let text = inner_text(&element).trim_matches('/').trim().to_owned();
            let text = if text.is_empty() { None } else { Some(text) };
            match i {
                0 => uk = text,
                1 => us = text,
                _ => unreachable!(),
            }
        }
        let pronunciation = Pronunciation { uk, us };

        let mut meanings = Vec::new();
        for element in self.0.select(&selectors::ec::MEANINGS) {
            let part_of_speech = element
                .select(&selectors::ec::PART_OF_SPEECH)
                .next()
                .map(|e| inner_text(&e));

            let definitions: Vec<String> = element
                .select(&selectors::ec::DEFINITIONS)
                .next()
                .map(|e| {
                    inner_text(&e)
                        .split('；')
                        .map(|s| s.trim().to_owned())
                        .collect()
                })
                .unwrap_or_default();

            if !definitions.is_empty() {
                meanings.push(Meaning {
                    part_of_speech,
                    definitions,
                });
            }
        }

        let exams: Vec<String> = self
            .0
            .select(&selectors::ec::EXAM_LIST)
            .next()
            .map(|container| {
                container
                    .select(&selectors::ec::EXAM)
                    .filter_map(|e| {
                        let t = inner_text(&e);
                        if t.is_empty() { None } else { Some(t) }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let examples: Vec<_> = example::extract_examples(&self.0)
            .into_iter()
            .map(|pair| Example {
                en: pair.source,
                zh: pair.target,
            })
            .collect();

        if examples.is_empty()
            && meanings.is_empty()
            && pronunciation.uk.is_none()
            && pronunciation.us.is_none()
        {
            return Err(Error::NoTranslationResults);
        }

        Ok(ToChinese {
            input_text,
            pronunciation,
            meanings,
            examples,
            exams,
        })
    }

    fn to_english(&self) -> std::result::Result<ToEnglish, Error> {
        let input_text = self.child_text(&selectors::common::WORD);

        let meanings: Vec<String> = self
            .0
            .select(&selectors::ce::POINT)
            .filter_map(|element| {
                let t = inner_text(&element);
                if t.is_empty() { None } else { Some(t) }
            })
            .collect();

        let examples: Vec<_> = example::extract_examples(&self.0)
            .into_iter()
            .map(|pair| Example {
                en: pair.target,
                zh: pair.source,
            })
            .collect();

        if examples.is_empty() && meanings.is_empty() {
            return Err(Error::NoTranslationResults);
        }

        Ok(ToEnglish {
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
    fn test_from_english() {
        let doc = Html::parse_document(include_str!("../fixtures/where_ec.html"));
        let dp = DictPage::new(doc.select(&crate::parse::selectors::BODY).next().unwrap());
        assert_eq!(
            <DictPage as EnglishParser>::to_chinese(&dp).unwrap(),
            ToChinese {
                input_text: "where".into(),
                pronunciation: Pronunciation {
                    uk: Some("weə(r)".into()),
                    us: Some("wer".into()),
                },
                meanings: vec![
                    Meaning {
                        part_of_speech: Some("adv.".into()),
                        definitions: vec!["在哪里".into()],
                    },
                    Meaning {
                        part_of_speech: Some("pron.".into()),
                        definitions: vec!["哪里".into()],
                    },
                    Meaning {
                        part_of_speech: Some("conj.".into()),
                        definitions: vec!["在……的地方".into()],
                    },
                    Meaning {
                        part_of_speech: Some("n.".into()),
                        definitions: vec!["地点".into()],
                    },
                ],
                examples: vec![
                    Example {
                        en: "Where were you yesterday morning?".into(),
                        zh: "你昨天上午在哪儿？".into(),
                    },
                    Example {
                        en: "Where were you at uni ?".into(),
                        zh: "你在哪儿上的大学？".into(),
                    },
                    Example {
                        en: "This is where I live.".into(),
                        zh: "这是我住的地方。".into(),
                    },
                ],
                exams: vec![
                    "初中".into(),
                    "高中".into(),
                    "CET4".into(),
                    "CET6".into(),
                    "考研".into()
                ],
            }
        );
    }

    #[test]
    fn test_to_english() {
        let doc = Html::parse_document(include_str!("../fixtures/where_ce.html"));
        let dp = DictPage::new(doc.select(&crate::parse::selectors::BODY).next().unwrap());
        assert_eq!(
            <DictPage as EnglishParser>::to_english(&dp).unwrap(),
            ToEnglish {
                input_text: "哪里".into(),
                meanings: vec!["where".into(), "wherever".into()],
                examples: vec![
                    Example {
                        en: "I'm free now to live wherever I please.".into(),
                        zh: "我现在想住哪里就住哪里。".into(),
                    },
                    Example {
                        en: "Where you want to go, Señorita?".into(),
                        zh: "你去哪里，女士？".into(),
                    },
                    Example {
                        en: "No one ordered him back whence he came.".into(),
                        zh: "没有人命令他从哪里来回哪里去。".into(),
                    },
                ],
            }
        );
    }
}
