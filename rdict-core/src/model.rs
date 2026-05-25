use crate::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Script {
    Chinese(String),
    English(String),
}

impl Script {
    pub fn classify(text: &str) -> Result<Self, Error> {
        if text.is_empty() {
            return Err(Error::EmptyInput);
        }
        if text
            .chars()
            .any(|ch| ('\u{4E00}'..='\u{9FFF}').contains(&ch))
        {
            Ok(Script::Chinese(text.to_owned()))
        } else {
            Ok(Script::English(text.to_owned()))
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Script::Chinese(s) | Script::English(s) => s,
        }
    }
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

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct NotFound {
    pub suggestions: Vec<String>,
}
