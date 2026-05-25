use clap::Parser;
use clap::ValueEnum;
use clap_complete::Shell;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliLanguage {
    #[value(alias = "en")]
    English,
    #[value(alias = "fr")]
    French,
    #[value(alias = "ja", alias = "jp")]
    Japanese,
    #[value(alias = "ko", alias = "kr")]
    Korean,
}

impl From<CliLanguage> for rdict_core::model::Language {
    fn from(lang: CliLanguage) -> Self {
        match lang {
            CliLanguage::English => Self::English,
            CliLanguage::French => Self::French,
            CliLanguage::Japanese => Self::Japanese,
            CliLanguage::Korean => Self::Korean,
        }
    }
}

#[derive(Parser)]
#[command(name = "rdict", version, about, long_about = None)]
pub struct Args {
    #[arg(value_name = "TEXT")]
    pub(crate) input_text: Vec<String>,

    /// Target language
    #[arg(long, value_enum, default_value = "english")]
    pub(crate) language: Option<CliLanguage>,

    /// Disable translation caches
    #[arg(long)]
    pub(crate) no_cache: bool,

    /// Output using JSON
    #[arg(long)]
    pub(crate) json: bool,

    /// Generate shell completions
    #[arg(long)]
    pub(crate) completion: Option<Shell>,
}
