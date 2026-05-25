use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "en")]
    #[default]
    English,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "ja")]
    Japanese,
}

impl Language {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
            Self::Korean => "ko",
            Self::Japanese => "ja",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::English => write!(f, "English"),
            Self::French => write!(f, "French"),
            Self::Korean => write!(f, "Korean"),
            Self::Japanese => write!(f, "Japanese"),
        }
    }
}
