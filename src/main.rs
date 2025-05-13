mod args;
mod parse;
mod rdict;

use crate::args::Args;
use crate::rdict::Rdict;
use anyhow::Context;
use clap::Parser;
use directories_next::ProjectDirs;
use rusqlite::Connection;
use std::io::{self, IsTerminal, Read};
use std::{fs, process};

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let stdin_is_piped = !io::stdin().is_terminal();

    let conn: Option<Connection> = if !cli.no_cache {
        if let Some(proj_dirs) = ProjectDirs::from("dev", "ny4", "rdict") {
            let cache_dir = proj_dirs.cache_dir();
            let db_path = cache_dir.join("cache.db");
            let should_init_db = !db_path.exists();

            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent).context("Failed to create cache directory")?;
            }

            let conn =
                Connection::open(&db_path).context("Failed to open database connection: {e}")?;

            if should_init_db {
                let init_statements = [
                    "
                    CREATE TABLE to_english_results (
                        word TEXT PRIMARY KEY,
                        data TEXT NOT NULL
                    );",
                    "
                    CREATE TABLE to_chinese_results (
                        word TEXT PRIMARY KEY,
                        data TEXT NOT NULL
                    );",
                ];

                for statement in &init_statements {
                    conn.execute(statement, ())
                        .context("Failed to initialize SQLite cache database: {e}")?;
                }
            }
            Some(conn)
        } else {
            eprintln!("Could not determine platform directories");
            process::exit(1);
        }
    } else {
        None
    };

    let rdict = Rdict::new("https://m.youdao.com", conn);

    match cli.word {
        Some(word) => rdict.output_results(&word)?,
        None if stdin_is_piped => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap();
            let word = buffer.trim();
            if word.is_empty() {
                eprintln!("No word specified.");
                process::exit(1);
            } else {
                rdict.output_results(word)?;
            }
        }
        _ => {
            rdict.interactive_mode().context("Interative mode failed")?;
        }
    }

    Ok(())
}
