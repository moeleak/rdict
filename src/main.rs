mod args;
mod parse;
mod rdict;

use crate::args::Args;
use crate::rdict::Rdict;
use clap::Parser;
use directories_next::ProjectDirs;
use rusqlite::Connection;
use std::io::{self, IsTerminal, Read};
use std::{fs, process};

fn main() {
    let cli = Args::parse();
    let stdin_is_piped = !io::stdin().is_terminal();

    let conn: Option<Connection> = if !cli.no_cache {
        if let Some(proj_dirs) = ProjectDirs::from("dev", "ny4", "rdict") {
            let cache_dir = proj_dirs.cache_dir();
            let db_path = cache_dir.join("cache.db");
            let should_init_db = !db_path.exists();

            if let Some(parent) = db_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create cache directory: {e}");
                    process::exit(1);
                }
            }

            match Connection::open(&db_path) {
                Ok(conn) => {
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
                            if let Err(e) = conn.execute(statement, ()) {
                                eprintln!("Failed to initialize SQLite cache database: {e}");
                                process::exit(1);
                            }
                        }
                    }
                    Some(conn)
                }
                Err(e) => {
                    eprintln!("Failed to open database connection: {e}");
                    process::exit(1);
                }
            }
        } else {
            eprintln!("Could not determine platform directories");
            process::exit(1);
        }
    } else {
        None
    };

    let rdict = Rdict::new("https://m.youdao.com", conn);

    match cli.word {
        Some(word) => rdict.output_results(&word),
        None if stdin_is_piped => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap();
            let word = buffer.trim();
            if word.is_empty() {
                eprintln!("No word specified.");
                process::exit(1);
            } else {
                rdict.output_results(word);
            }
        }
        _ => {
            if let Err(e) = rdict.interactive_mode() {
                eprintln!("Interactive mode failed: {e:?}");
                process::exit(1);
            }
        }
    }
}
