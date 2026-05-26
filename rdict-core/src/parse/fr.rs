use crate::Error;
use crate::parse::{DictPage, example, inner_text};
use scraper::Selector;
use serde::{Deserialize, Serialize};

pub trait FrenchParser {
    fn to_chinese(&self) -> std::result::Result<ToChinese, Error>;
    fn to_french(&self) -> std::result::Result<ToFrench, Error>;
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Example {
    pub fr: String,
    pub zh: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToChinese {
    pub input_text: String,
    pub pronunciation: Option<String>,
    pub meanings: Vec<String>,
    pub examples: Vec<Example>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToFrench {
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

        pub static WORD:        LazyLock<Selector> = selector!("div.dict-module h4");
    }

    pub mod fc {
        use super::*;

        pub static PHONETIC:    LazyLock<Selector> = selector!("div.dict-module div.phone_con span.phonetic");
        pub static DEFINITION:  LazyLock<Selector> = selector!("div.dict-module p.pos");
    }

    pub mod cf {
        use super::*;

        pub static POINT:       LazyLock<Selector> = selector!("div.dict-module li.each-word a.point");
    }
}

impl FrenchParser for DictPage<'_> {
    fn to_chinese(&self) -> std::result::Result<ToChinese, Error> {
        let input_text = self.child_text(&selectors::common::WORD);

        let pronunciation = self
            .0
            .select(&selectors::fc::PHONETIC)
            .next()
            .map(|el| inner_text(&el).trim_matches('/').trim().to_owned())
            .filter(|s| !s.is_empty());

        let meanings: Vec<String> = self
            .0
            .select(&selectors::fc::DEFINITION)
            .flat_map(|el| {
                let text = inner_text(&el);
                if text.is_empty() {
                    vec![]
                } else {
                    text.split('；')
                        .map(|s| s.trim().to_owned())
                        .filter(|s| !s.is_empty())
                        .collect()
                }
            })
            .collect();

        if meanings.is_empty() {
            return Err(Error::NoTranslationResults);
        }

        let examples = example::extract_examples(&self.0)
            .into_iter()
            .map(|pair| Example {
                fr: pair.source,
                zh: pair.target,
            })
            .collect();

        Ok(ToChinese {
            input_text,
            pronunciation,
            meanings,
            examples,
        })
    }

    fn to_french(&self) -> std::result::Result<ToFrench, Error> {
        let input_text = self.child_text(&selectors::common::WORD);

        let meanings: Vec<String> = self
            .0
            .select(&selectors::cf::POINT)
            .filter_map(|el| {
                let t = inner_text(&el);
                if t.is_empty() { None } else { Some(t) }
            })
            .collect();

        if meanings.is_empty() {
            return Err(Error::NoTranslationResults);
        }

        let examples = example::extract_examples(&self.0)
            .into_iter()
            .map(|pair| Example {
                fr: pair.target,
                zh: pair.source,
            })
            .collect();

        Ok(ToFrench {
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
    fn test_from_french() {
        let doc = Html::parse_document(include_str!("../fixtures/where_fc.html"));
        let dp = DictPage::new(doc.select(&crate::parse::selectors::BODY).next().unwrap());
        assert_eq!(
            <DictPage as FrenchParser>::to_chinese(&dp).unwrap(),
            ToChinese {
                input_text: "où".into(),
                pronunciation: Some("u".into()),
                meanings: vec![
                    "在那里，在那个地方".into(),
                    "在哪儿，去哪里".into(),
                    "在…地方，去…地方".into(),
                    "在…状态".into(),
                ],
                examples: vec![
                    Example {
                        fr: "Où sont ses livres et journaux.".into(),
                        zh: "他的书和报纸在哪儿？".into(),
                    },
                    Example {
                        fr: "Où est la bibliothèque?".into(),
                        zh: "图书馆在哪儿？".into(),
                    },
                    Example {
                        fr: "Où?a? je ne vois rien.".into(),
                        zh: "在哪啊？我什么都没看到。".into(),
                    },
                ],
            }
        );
    }

    #[test]
    fn test_to_french() {
        let doc = Html::parse_document(include_str!("../fixtures/where_cf.html"));
        let dp = DictPage::new(doc.select(&crate::parse::selectors::BODY).next().unwrap());
        assert_eq!(
            <DictPage as FrenchParser>::to_french(&dp).unwrap(),
            ToFrench {
                input_text: "哪里".into(),
                meanings: vec!["où".into()],
                examples: vec![
                    Example {
                        fr: "Ou est la station de metro?".into(),
                        zh: "请问地铁车站在哪里？".into(),
                    },
                    Example {
                        fr: "Où se trouve votre province?".into(),
                        zh: "您出生的省份在哪里？".into(),
                    },
                    Example {
                        fr: "Où est passé mon instinct?".into(),
                        zh: "哪里是我过去的本能？".into(),
                    },
                ],
            }
        );
    }
}
