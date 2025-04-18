mod args;
mod parse;
mod rdict;

use crate::args::Args;
use crate::rdict::Rdict;
use clap::Parser;
use std::io::{self, IsTerminal, Read};
use std::process;

fn main() {
    let cli = Args::parse();
    let stdin_is_piped = !io::stdin().is_terminal();

    let rdict = Rdict::new("https://m.youdao.com");

    match cli.word {
        Some(word) => rdict.output_results(&word),
        None if stdin_is_piped => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap();
            let word = buffer.trim();
            if !word.is_empty() {
                rdict.output_results(word);
            } else {
                eprintln!("No word specified.");
                process::exit(1);
            }
        }
        _ => {
            if let Err(e) = rdict.interactive_mode() {
                eprintln!("Interactive mode failed: {:?}", e);
                process::exit(1);
            }
        }
    }
}
