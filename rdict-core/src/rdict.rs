use crate::Error;
use crate::parse::{
    ToChinese, ToEnglish, TranslationData, not_found, selectors, to_chinese, to_english,
};
use log::{debug, info};
use owo_colors::OwoColorize;
use reqwest::Client;
use scraper::Html;
use sqlx::Row;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePool;
use std::fmt::Write;
use std::fs;

type CacheVariant = (&'static str, fn(String) -> Result<TranslationData, Error>);

#[derive(Debug, Clone)]

pub struct Rdict {
    client: Client,
    base_url: String,
    pool: Option<SqlitePool>,
}

#[derive(Debug)]
pub enum Format {
    /// Markdown with ANSI color escape sequences
    MarkdownColored,
    /// Plain Markdown
    Markdown,
    /// Formatted JSON
    Json,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FetchedResult {
    pub data: TranslationData,
    pub is_cached: bool,
}

impl Rdict {
    pub async fn new(
        base_url: &str,
        cache_db_path: Option<std::path::PathBuf>,
    ) -> Result<Self, Error> {
        let pool: Option<SqlitePool> = if let Some(db_path) = cache_db_path {
            let should_init_db = !db_path.exists();

            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let db_url = db_path
                .to_str()
                .ok_or_else(|| Error::InvalidDatabasePath(db_path.clone()))?;

            if !sqlx::Sqlite::database_exists(db_url).await? {
                sqlx::Sqlite::create_database(db_url).await?;
            }

            let pool = SqlitePool::connect(db_url).await?;

            if should_init_db {
                let init_statements = [
                    "CREATE TABLE to_english_results (
                        text TEXT PRIMARY KEY,
                        data TEXT NOT NULL
                    );",
                    "CREATE TABLE to_chinese_results (
                        text TEXT PRIMARY KEY,
                        data TEXT NOT NULL
                    );",
                ];

                for statement in init_statements {
                    sqlx::query(statement).execute(&pool).await?;
                }
            }

            Some(pool)
        } else {
            None
        };

        Ok(Self {
            client: Client::new(),
            base_url: base_url.to_owned(),
            pool,
        })
    }

    pub async fn get_results(&self, input_text: &str) -> Result<FetchedResult, Error> {
        let is_cjk = contains_cjk(input_text)?;
        debug!("Getting result. input_text: {input_text}, is_cjk: {is_cjk}");

        // Try cache first
        if let Some(pool) = &self.pool {
            let (table, variant): CacheVariant = if is_cjk {
                debug!("Caching enabled, trying fetch data from cache.");
                ("to_english_results", |v| {
                    Ok(TranslationData::ToEnglish(serde_json::from_str(&v)?))
                })
            } else {
                ("to_chinese_results", |v| {
                    Ok(TranslationData::ToChinese(serde_json::from_str(&v)?))
                })
            };

            let query = format!("SELECT data FROM {table} WHERE text = ?");
            let delete_row = async || {
                let query = format!("DELETE FROM {table} WHERE text = ?");

                sqlx::query(&query).bind(input_text).execute(pool).await
            };

            match sqlx::query(&query)
                .bind(input_text)
                .fetch_optional(pool)
                .await
            {
                Ok(Some(result)) => {
                    let data_str: String = result.try_get("data")?;
                    match variant(data_str) {
                        Ok(data) => {
                            return Ok(FetchedResult {
                                data,
                                is_cached: true,
                            });
                        }
                        Err(_) => {
                            delete_row().await?;
                        }
                    }
                }

                Ok(None) => {
                    info!("Translation cache missed, fetching translation data again.");
                }

                Err(_) => {
                    info!(
                        "Database error, deleting row from SQLite cache and fetching translation data again."
                    );
                    delete_row().await?;
                }
            }
        }

        // Fetch from web
        let html = self.fetch_text_html(input_text).await?;

        let (result, data_for_cache): (TranslationData, Option<String>) = {
            let binding = Html::parse_document(&html);
            let document = binding
                .select(&selectors::BODY_SELECTOR)
                .next()
                .ok_or(Error::Parse("no .search_result-dict found".into()))?;

            if is_cjk {
                let result = match to_english(input_text, document) {
                    Ok(translation) => translation,

                    Err(Error::NoTranslationResults) => {
                        let nf_data = not_found(document)?;

                        return Ok(FetchedResult {
                            data: TranslationData::NotFound(nf_data),
                            is_cached: false,
                        });
                    }

                    Err(other_error) => return Err(other_error),
                };

                let data = self
                    .pool
                    .as_ref()
                    .map(|_| serde_json::to_string(&result))
                    .transpose()?;
                (TranslationData::ToEnglish(result), data)
            } else {
                let result = match to_chinese(input_text, document) {
                    Ok(translation) => translation,

                    Err(Error::NoTranslationResults) => {
                        let nf_data = not_found(document)?;

                        return Ok(FetchedResult {
                            data: TranslationData::NotFound(nf_data),
                            is_cached: false,
                        });
                    }

                    Err(other_error) => return Err(other_error),
                };

                let data = self
                    .pool
                    .as_ref()
                    .map(|_| serde_json::to_string(&result))
                    .transpose()?;
                (TranslationData::ToChinese(result), data)
            }
        };

        if let Some(pool) = &self.pool
            && let Some(data) = data_for_cache
        {
            let query = if is_cjk {
                "INSERT OR REPLACE INTO to_english_results (text, data) VALUES (?, ?)"
            } else {
                "INSERT OR REPLACE INTO to_chinese_results (text, data) VALUES (?, ?)"
            };

            sqlx::query(query)
                .bind(input_text)
                .bind(&data)
                .execute(pool)
                .await?;
        }

        Ok(FetchedResult {
            data: result,
            is_cached: false,
        })
    }

    async fn fetch_text_html(&self, text: &str) -> Result<String, reqwest::Error> {
        let url = format!("{}/result", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[("word", text), ("lang", "en")])
            .header(
                reqwest::header::USER_AGENT,
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
            )
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }
}

fn contains_cjk(text: &str) -> Result<bool, Error> {
    if text.is_empty() {
        return Err(Error::EmptyInput);
    }
    Ok(text
        .chars()
        .any(|ch| ('\u{4E00}'..='\u{9FFF}').contains(&ch)))
}

impl ToChinese {
    #[must_use]
    pub fn render_colored(&self) -> String {
        let mut output = String::new();

        if self.pronunciation.uk.is_some() || self.pronunciation.us.is_some() {
            writeln!(output, "{}", "# Pronunciation".bright_black()).unwrap();

            if let Some(ref uk) = self.pronunciation.uk {
                writeln!(output, "英：[{}]", uk.green()).unwrap();
            }

            if let Some(ref us) = self.pronunciation.us {
                writeln!(output, "美：[{}]", us.green()).unwrap();
            }

            writeln!(output).unwrap();
        }

        if !self.meanings.is_empty() {
            writeln!(output, "{}", "# Meanings".bright_black()).unwrap();
            for me in &self.meanings {
                if let Some(ref pa) = me.part_of_speech {
                    writeln!(output, "[{pa}]").unwrap();
                }
                for de in &me.definitions {
                    writeln!(output, "* {}", de.green()).unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        if !self.examples.is_empty() {
            writeln!(output, "{}", "# Examples".bright_black()).unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.en.green()).unwrap();
                writeln!(output, "  {}", ex.zh.magenta()).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }

    #[must_use]
    pub fn render_plain(&self) -> String {
        let mut output = String::new();

        if self.pronunciation.uk.is_some() || self.pronunciation.us.is_some() {
            writeln!(output, "# Pronunciation").unwrap();

            if let Some(ref uk) = self.pronunciation.uk {
                writeln!(output, "英：[{uk}]").unwrap();
            }

            if let Some(ref us) = self.pronunciation.us {
                writeln!(output, "美：[{us}]").unwrap();
            }

            writeln!(output).unwrap();
        }

        if !self.meanings.is_empty() {
            writeln!(output, "# Meanings").unwrap();
            for me in &self.meanings {
                if let Some(ref pa) = me.part_of_speech {
                    writeln!(output, "[{pa}]").unwrap();
                }
                for de in &me.definitions {
                    writeln!(output, "* {de}").unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        if !self.examples.is_empty() {
            writeln!(output, "# Examples").unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.en).unwrap();
                writeln!(output, "  {}", ex.zh).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }
}

impl ToEnglish {
    #[must_use]
    pub fn render_colored(&self) -> String {
        let mut output = String::new();

        if !self.meanings.is_empty() {
            writeln!(output, "{}", "# Meanings".bright_black()).unwrap();
            for me in &self.meanings {
                writeln!(output, "* {}", me.green()).unwrap();
            }
            writeln!(output).unwrap();
        }

        if !self.examples.is_empty() {
            writeln!(output, "{}", "# Examples".bright_black()).unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.en.green()).unwrap();
                writeln!(output, "  {}", ex.zh.magenta()).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }

    #[must_use]
    pub fn render_plain(&self) -> String {
        let mut output = String::new();

        if !self.meanings.is_empty() {
            writeln!(output, "# Meanings").unwrap();
            for me in &self.meanings {
                writeln!(output, "* {me}").unwrap();
            }
            writeln!(output).unwrap();
        }

        if !self.examples.is_empty() {
            writeln!(output, "# Examples").unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.en).unwrap();
                writeln!(output, "  {}", ex.zh).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{Example, Meaning, Pronunciation};
    use mockito::{Matcher, Server};

    #[test]
    fn test_contains_cjk_with_cjk() {
        assert!(contains_cjk("你好").unwrap());
    }

    #[test]
    fn test_contains_cjk_without_cjk() {
        assert!(!contains_cjk("hello").unwrap());
    }

    #[test]
    fn test_contains_cjk_mixed_input() {
        assert!(contains_cjk("hello你好").unwrap());
    }

    #[test]
    fn test_contains_cjk_empty() {
        contains_cjk("").unwrap_err();
    }

    #[tokio::test]
    #[expect(clippy::significant_drop_tightening)]
    async fn test_fetch_text_html_success_with_mock_server() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/result")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("word".into(), "hello".into()),
                Matcher::UrlEncoded("lang".into(), "en".into()),
            ]))
            .with_status(200)
            .with_body(include_str!("fixtures/hello_response.html"))
            .create();

        let client = Rdict::new(&server.url(), None).await.unwrap();

        let html = client.fetch_text_html("hello").await.unwrap();
        assert!(html.contains("Hello"));
        mock.assert();
    }

    #[tokio::test]
    async fn test_translation() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/result")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("word".into(), "hello".into()),
                Matcher::UrlEncoded("lang".into(), "en".into()),
            ]))
            .with_status(200)
            .with_body(include_str!("fixtures/hello_response.html"))
            .create();

        let client = Rdict::new(&server.url(), None).await.unwrap();

        let result = client.get_results("hello").await.unwrap();

        println!("{:?}", result);

        assert_eq!(
            result,
            FetchedResult {
                is_cached: false,
                data: TranslationData::ToChinese(ToChinese {
                    input_text: "hello".to_owned(),
                    pronunciation: Pronunciation {
                        uk: Some("həˈləʊ".to_owned()),
                        us: Some("həˈloʊ".to_owned()),
                    },
                    meanings: vec![
                        Meaning {
                            part_of_speech: Some("int.".to_owned()),
                            definitions: vec![
                                "喂，你好（用于问候或打招呼）".to_owned(),
                                "喂，你好（打电话时的招呼语）".to_owned(),
                                "喂，你好（引起别人注意的招呼语）".to_owned(),
                                "<非正式>喂，嘿 (认为别人说了蠢话或分心)".to_owned(),
                                "<英，旧>嘿（表示惊讶）".to_owned(),
                            ],
                        },
                        Meaning {
                            part_of_speech: Some("n.".to_owned()),
                            definitions: vec![
                                "招呼，问候".to_owned(),
                                "（Hello）（法、印、美、俄）埃洛（人名）".to_owned(),
                            ],
                        },
                        Meaning {
                            part_of_speech: Some("v.".to_owned()),
                            definitions: vec!["说（或大声说）“喂”".to_owned(), "打招呼".to_owned(),],
                        },
                    ],
                    examples: vec![
                        Example {
                            en: "'Hello, Paul,' they chorused.".to_owned(),
                            zh: "“你好，保罗。”他们齐声问候道。".to_owned(),
                        },
                        Example {
                            en: "Hello, is there anybody there?".to_owned(),
                            zh: "喂，那里有人吗？".to_owned(),
                        },
                        Example {
                            en: "Hello, is Gordon there please?".to_owned(),
                            zh: "您好，请问戈登在吗？".to_owned(),
                        },
                    ],
                    exams: vec![
                        "初中".to_owned(),
                        "高中".to_owned(),
                        "CET4".to_owned(),
                        "CET6".to_owned(),
                        "考研".to_owned()
                    ],
                }),
            }
        );

        mock.assert();
    }
}
