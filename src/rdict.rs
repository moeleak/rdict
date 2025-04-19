use crate::parse::{ToChinese, ToEnglish, to_chinese, to_english};
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
    conn: Option<rusqlite::Connection>,
}

impl Rdict {
    pub fn new(base_url: &str, conn: Option<rusqlite::Connection>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            conn,
        }
    }

    pub fn output_results(&self, word: &str) {
        let is_cjk = Self::contains_cjk(word);

        // Retrieve from cache if available
        if let Some(conn) = &self.conn {
            match is_cjk {
                true => {
                    let mut stmt = conn
                        .prepare("SELECT data FROM to_english_results WHERE word = ?1")
                        .unwrap();
                    let mut rows = stmt.query([word]).unwrap();
                    if let Ok(Some(row)) = rows.next() {
                        let data: String = row.get(0).unwrap();
                        if let Ok(result) = serde_json::from_str::<ToEnglish>(&data) {
                            self.output_english(word, result, false);
                            return;
                        }
                    }
                }
                false => {
                    let mut stmt = conn
                        .prepare("SELECT data FROM to_chinese_results WHERE word = ?1")
                        .unwrap();
                    let mut rows = stmt.query([word]).unwrap();
                    if let Some(row) = rows.next().unwrap() {
                        let data: String = row.get(0).unwrap();
                        if let Ok(result) = serde_json::from_str::<ToChinese>(&data) {
                            self.output_chinese(word, result, false);
                            return;
                        }
                    }
                }
            }
        }


        // If not found in cache, fetch from the web
        let word_html = self.fetch_word_html(word);
        match word_html {
            Ok(html) => {
                if is_cjk {
                    match to_english(&html) {
                        Ok(result) => {
                            self.output_english(word, result, true);
                        }
                        Err(e) => {
                            eprintln!("Error: {e}");
                        }
                    }
                } else {
                    match to_chinese(&html) {
                        Ok(result) => {
                            self.output_chinese(word, result, true);
                        }
                        Err(e) => {
                            eprintln!("Error: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error fetching HTML: {e}");
            }
        }
    }

    pub fn output_english(&self, word: &str, result: ToEnglish, save_to_cache: bool) {
        if save_to_cache {
            if let Some(conn) = &self.conn {
                match serde_json::to_string(&result) {
                    Ok(data) => {
                        if let Err(e) = conn.execute(
                    "INSERT OR REPLACE INTO to_english_results (word, data) VALUES (?1, ?2)",
                    rusqlite::params![word, data],
                ) {
                    eprintln!("Error saving to cache: {e}");
                }
                    }
                    Err(e) => {
                        eprintln!("Error serializing result: {e}");
                    }
                }
            }
        }

        let mut output = "\n".to_string();

        output += &result.translations.join("; ").green().to_string();
        output += "\n\n";

        for (i, example) in result.example_sentenses.iter().enumerate() {
            output += &format!("  {}.{}", i + 1, example.english_sentense)
                .green()
                .to_string();
            output += "\n";
            output += &format!("    {}", example.chinese_sentense)
                .magenta()
                .to_string();
            output += "\n";
        }

        println!("{}", output);
    }

    pub fn output_chinese(&self, word: &str, result: ToChinese, save_to_cache: bool) {
        if save_to_cache {
            if let Some(conn) = &self.conn {
                match serde_json::to_string(&result) {
                    Ok(data) => {
                        if let Err(e) = conn.execute(
                    "INSERT OR REPLACE INTO to_chinese_results (word, data) VALUES (?1, ?2)",
                    rusqlite::params![word, data],
                ) {
                    eprintln!("Error saving to cache: {e}");
                }
                    }
                    Err(e) => {
                        eprintln!("Error serializing result: {e}");
                    }
                }
            }
        }

        let mut output = "\n".to_string();

        output += &format!("    英：{} 美：{}", result.phonetic.uk, result.phonetic.us)
            .green()
            .to_string();
        output += "\n\n";

        for translation in result.translations {
            output += &format!(
                "      {} {}",
                translation.english_word_type, translation.chinese_translation
            )
            .green()
            .to_string();
            output += "\n";
        }
        output += "\n";

        for (i, example) in result.example_sentenses.iter().enumerate() {
            output += &format!("  {}.{}", i + 1, example.english_sentense)
                .green()
                .to_string();
            output += "\n";

            output += &format!("    {}", example.chinese_sentense)
                .magenta()
                .to_string();
            output += "\n";
        }

        println!("{}", output);
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

    pub fn contains_cjk(word: &str) -> bool {
        let re = Regex::new(r"[\p{Han}]").unwrap();
        re.is_match(word)
    }
}
