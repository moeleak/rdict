use clap::Parser;
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "rdict", version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(value_name = "TEXT")]
    pub(crate) input_text: Option<String>,

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
