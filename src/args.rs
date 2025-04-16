use clap::Parser;

use clap_stdin::MaybeStdin;

#[derive(Debug, Parser)]
pub struct Args {
    pub word: MaybeStdin<String>,
}
