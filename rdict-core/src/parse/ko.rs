use crate::Error;
use crate::parse::{DictPage, example, inner_text};
use scraper::Selector;
use serde::{Deserialize, Serialize};

pub trait KoreanParser {
    fn to_chinese(&self) -> std::result::Result<ToChinese, Error>;
    fn to_korean(&self) -> std::result::Result<ToKorean, Error>;
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToChinese {
    pub input_text: String,
    pub meanings: Vec<Meaning>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Meaning {
    pub part_of_speech: Option<String>,
    pub definitions: Vec<String>,
    pub example: Option<Example>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Example {
    pub ko: String,
    pub zh: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToKorean {
    pub input_text: String,
    pub meanings: Vec<Meaning>,
    pub examples: Vec<Example>,
}

#[rustfmt::skip]
mod selectors {
    use std::sync::LazyLock;
    use crate::selector;
    use super::Selector;

    pub mod common {
        use super::*;

        pub static WORD:            LazyLock<Selector> = selector!(".word-head div.title");
    }

    pub mod kc {
        use super::*;

        pub static MEANING:         LazyLock<Selector> = selector!("ul.tran-cont > li.mcols");
        pub static PART_OF_SPEECH:  LazyLock<Selector> = selector!("span.kcPos");
        pub static KEY:             LazyLock<Selector> = selector!("span.kcKey");
        pub static EXAMPLE:         LazyLock<Selector> = selector!("li.secondary");
        pub static EXAMPLE_P:       LazyLock<Selector> = selector!("p");
    }

    pub mod ck {
        use super::*;

        pub static KEY:             LazyLock<Selector> = selector!("ul.tran-cont li.mcols p.ckKey");
        pub static MEANING_GROUP:   LazyLock<Selector> = selector!("div.trans-container > ul.tran-cont");
        pub static PART_OF_SPEECH:  LazyLock<Selector> = selector!("div.trans-container > div.pos");
    }
}

impl KoreanParser for DictPage<'_> {
    fn to_chinese(&self) -> std::result::Result<ToChinese, Error> {
        let input_text = self.child_text(&selectors::common::WORD);

        let mut meanings = Vec::new();
        for li in self.0.select(&selectors::kc::MEANING) {
            let part_of_speech = li
                .select(&selectors::kc::PART_OF_SPEECH)
                .next()
                .map(|el| {
                    inner_text(&el)
                        .trim_matches('【')
                        .trim_matches('】')
                        .to_owned()
                })
                .filter(|s| !s.is_empty());

            let raw = li
                .select(&selectors::kc::KEY)
                .next()
                .map(|el| inner_text(&el))
                .unwrap_or_default();

            let definitions: Vec<String> = raw
                .split(|c: char| c.is_ascii_digit() && c != '0')
                .filter_map(|s| {
                    let trimmed = s.trim().trim_start_matches('.');
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_owned())
                    }
                })
                .collect();

            let example = li.select(&selectors::kc::EXAMPLE).next().and_then(|el| {
                let paragraphs: Vec<String> = el
                    .select(&selectors::kc::EXAMPLE_P)
                    .map(|p| inner_text(&p))
                    .collect();

                if paragraphs.len() >= 2 {
                    Some(Example {
                        ko: paragraphs[0].clone(),
                        zh: paragraphs[1].clone(),
                    })
                } else {
                    None
                }
            });

            if !definitions.is_empty() {
                meanings.push(Meaning {
                    part_of_speech,
                    definitions,
                    example,
                });
            }
        }

        if meanings.is_empty() {
            return Err(Error::NoTranslationResults);
        }

        Ok(ToChinese {
            input_text,
            meanings,
        })
    }

    fn to_korean(&self) -> std::result::Result<ToKorean, Error> {
        let input_text = self.child_text(&selectors::common::WORD);

        let pos_elements: Vec<_> = self.0.select(&selectors::ck::PART_OF_SPEECH).collect();
        let mut meanings = Vec::new();
        for (i, group) in self.0.select(&selectors::ck::MEANING_GROUP).enumerate() {
            let part_of_speech = if i > 0 {
                pos_elements
                    .get(i - 1)
                    .map(|el| {
                        inner_text(el)
                            .trim_matches('[')
                            .trim_matches(']')
                            .to_owned()
                    })
                    .filter(|s| !s.is_empty())
            } else {
                None
            };

            let definitions: Vec<String> = group
                .select(&selectors::ck::KEY)
                .flat_map(|el| {
                    inner_text(&el)
                        .split('；')
                        .map(|s| s.trim().to_owned())
                        .collect::<Vec<_>>()
                })
                .filter(|s| !s.is_empty())
                .collect();

            if !definitions.is_empty() {
                meanings.push(Meaning {
                    part_of_speech,
                    definitions,
                    example: None,
                });
            }
        }

        if meanings.is_empty() {
            return Err(Error::NoTranslationResults);
        }

        let examples = example::extract_examples(&self.0)
            .into_iter()
            .map(|pair| Example {
                ko: pair.target,
                zh: pair.source,
            })
            .collect();

        Ok(ToKorean {
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
    fn test_from_korean() {
        let doc = Html::parse_document(include_str!("../fixtures/where_kc.html"));
        let dp = DictPage::new(doc.select(&crate::parse::selectors::BODY).next().unwrap());
        assert_eq!(
            <DictPage as KoreanParser>::to_chinese(&dp).unwrap(),
            ToChinese {
                input_text: "어디".into(),
                meanings: vec![
                    Meaning {
                        part_of_speech: Some("代名词".into()),
                        definitions: vec![
                            "(指示代词)哪里。哪儿。".into(),
                            "指难言的什么或什么地方。".into(),
                        ],
                        example: Some(Example {
                            ko: "어디서 오느냐?".into(),
                            zh: "从何而来？".into(),
                        }),
                    },
                    Meaning {
                        part_of_speech: Some("感叹词".into()),
                        definitions: vec!["表示强调反问的话。".into(), "表示强调某种意志。".into(),],
                        example: Some(Example {
                            ko: "또 어떤 이유가 있는지, 내가 어디 들어 보겠다.".into(),
                            zh: "还有什么理由，我倒是想听一听。".into(),
                        }),
                    },
                ],
            }
        );
    }

    #[test]
    fn test_to_korean() {
        let doc = Html::parse_document(include_str!("../fixtures/where_ck.html"));
        let dp = DictPage::new(doc.select(&crate::parse::selectors::BODY).next().unwrap());
        assert_eq!(
            <DictPage as KoreanParser>::to_korean(&dp).unwrap(),
            ToKorean {
                input_text: "哪里".into(),
                meanings: vec![
                    Meaning {
                        part_of_speech: None,
                        definitions: vec!["어찌하여. 어떻게.".into()],
                        example: None,
                    },
                    Meaning {
                        part_of_speech: Some("代名词".into()),
                        definitions: vec!["어디".into(), "어느 곳".into()],
                        example: None,
                    },
                    Meaning {
                        part_of_speech: Some("词组/表达".into()),
                        definitions: vec!["별말씀을요".into(), "천만에요".into()],
                        example: None,
                    },
                ],
                examples: vec![
                    Example {
                        ko: "어디서 넘어지든, 넘어진 자리에서 바로 일떠서다[기운차게 일어나다]."
                            .into(),
                        zh: "在哪里跌倒，就在哪里爬起来。".into(),
                    },
                    Example {
                        ko: "어디서 넘어지든 바로 그 자리에서 일어난다.".into(),
                        zh: "在哪里跌倒，就在哪里爬起来。".into(),
                    },
                    Example {
                        ko: "그는 어디 잇속이 있다하면, 바로 거기에 개입한다.".into(),
                        zh: "哪里有利可图，他就往哪里伸腿。".into(),
                    },
                ],
            }
        );
    }
}
