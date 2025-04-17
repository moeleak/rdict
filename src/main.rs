mod args;
mod parse;
use crate::args::Args;
use crate::parse::to_chinese;
use crate::parse::to_english;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use regex::Regex;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    let cli = Args::parse();
    let word: String = cli.word.to_string();

    match fetch_word_html(&word) {
        Ok(html) => {
            let is_cjk = contains_cjk(&word);

            if is_cjk {
                match to_english(&html) {
                    Ok(result) => {
                        println!("\n{}", result.translations.join("; ").green());

                        for (i, example) in result.example_sentenses.iter().enumerate() {
                            println!(
                                "{}",
                                format!("  {}.{}", i + 1, example.english_sentense).green()
                            );
                            println!("    {}", example.chinese_sentense.magenta());
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching HTML: {}", e);
                    }
                };
            } else {
                match to_chinese(&html) {
                    Ok(result) => {
                        println!(
                            "{}",
                            format!(
                                "\n    英：{} 美：{}\n\n",
                                result.phonetic.uk, result.phonetic.us
                            )
                            .green()
                        );

                        for translation in result.translations {
                            println!(
                                "{}",
                                format!(
                                    "      {} {}",
                                    translation.english_word_type, translation.chinese_translation
                                )
                                .green()
                            )
                        }

                        println!("\n");

                        for (i, example) in result.example_sentenses.iter().enumerate() {
                            println!(
                                "{}",
                                format!("  {}.{}", i + 1, example.english_sentense).green()
                            );
                            println!("    {}", example.chinese_sentense.magenta());
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching HTML: {}", e);
                    }
                };
            }
        }
        Err(e) => {
            eprintln!("Error fetching HTML: {}", e);
        }
    };
}

fn fetch_word_html(word: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();

    let base_url = "https://m.youdao.com/result";
    let mut params = HashMap::new();
    params.insert("word", word);
    params.insert("lang", "en");

    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Fetching data...");
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner} {msg}")
            .unwrap(),
    );

    let response = client
        .get(base_url)
        .query(&params)
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
        )
        .send()?
        .text()?;

    Ok(response)
}

fn contains_cjk(word: &str) -> bool {
    let re = Regex::new(r"[\p{Han}]").unwrap();
    re.is_match(word)
}
