use clap::Parser;

use clap_stdin::MaybeStdin;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub word: MaybeStdin<String>,
}
