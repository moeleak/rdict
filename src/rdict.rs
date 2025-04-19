use crate::parse::{to_chinese, to_english};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use regex::Regex;
use reqwest::blocking::Client;
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::time::Duration;

pub struct Rdict {
    client: Client,
    base_url: String,
}

impl Rdict {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub fn output_results(&self, word: &str) {
        let word_html = Self::fetch_word_html(self, word);
        let is_cjk = Self::contains_cjk(word);

        match word_html {
            Ok(html) => {
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
                            eprintln!("Error fetching HTML: {e}");
                        }
                    }
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
                                        translation.english_word_type,
                                        translation.chinese_translation
                                    )
                                    .green()
                                );
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
                            eprintln!("Error fetching HTML: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error fetching HTML: {e}");
            }
        }
    }

    pub fn interactive_mode(&self) -> rustyline::Result<()> {
        let mut rl = DefaultEditor::new()?;
        loop {
            let readline = rl.readline(format!("{}# ", "[rdict]".green()).as_str());
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str())?;
                    let word = line.as_str().trim();
                    Self::output_results(self, word);
                }
                Err(err) => {
                    println!("Error: {err:?}");
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn fetch_word_html(&self, word: &str) -> Result<String, reqwest::Error> {
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

        let url = format!("{}/result", self.base_url);
        let response = self
            .client
            .get(&url)
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

    pub fn contains_cjk(word: &str) -> bool {
        let re = Regex::new(r"[\p{Han}]").unwrap();
        re.is_match(word)
    }
}
