use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub word: Option<String>,
}
