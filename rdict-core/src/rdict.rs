use crate::Error;
use crate::model::{NotFound, Script, ToChinese, ToEnglish};
use crate::parse::{DictPage, selectors};
use log::{debug, info};
use reqwest::Client;
use scraper::Html;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePool;
use std::fs;

#[derive(Debug, Clone)]

pub struct Rdict {
    client: Client,
    base_url: String,
    pool: Option<SqlitePool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FetchedResult {
    pub data: TranslationData,
    pub is_cached: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum TranslationData {
    #[serde(rename = "to_chinese")]
    ToChinese(ToChinese),

    #[serde(rename = "to_english")]
    ToEnglish(ToEnglish),

    #[serde(rename = "not_found")]
    NotFound(NotFound),
}

impl TranslationData {
    fn as_render(&self) -> &dyn crate::render::Render {
        match self {
            TranslationData::ToChinese(x) => x,
            TranslationData::ToEnglish(x) => x,
            TranslationData::NotFound(x) => x,
        }
    }

    pub fn render_colored(&self) -> String {
        self.as_render().render_colored()
    }

    pub fn render_plain(&self) -> String {
        self.as_render().render_plain()
    }
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
        let script = Script::classify(input_text)?;
        debug!(
            "Getting result. input_text: {}, script: {:?}",
            input_text, script
        );

        if let Some(result) = self.try_cache(&script).await? {
            return Ok(result);
        }

        let data = self.fetch_and_parse(&script).await?;

        if let Some(pool) = &self.pool {
            let serialized = serde_json::to_string(&data)?;
            self.write_cache(pool, &script, &serialized).await?;
        }

        Ok(FetchedResult {
            data,
            is_cached: false,
        })
    }

    async fn try_cache(&self, script: &Script) -> Result<Option<FetchedResult>, Error> {
        let pool = match &self.pool {
            Some(p) => p,
            None => return Ok(None),
        };

        debug!("Caching enabled, trying fetch data from cache.");

        let table = match script {
            Script::Chinese(_) => "to_english_results",
            Script::English(_) => "to_chinese_results",
        };

        let select_query = format!("SELECT data FROM {table} WHERE text = ?");
        let delete_query = format!("DELETE FROM {table} WHERE text = ?");

        let row_result = sqlx::query(&select_query)
            .bind(script.text())
            .fetch_optional(pool)
            .await;

        let row = match row_result {
            Ok(Some(row)) => row,
            Ok(None) => {
                info!("Translation cache missed, fetching translation data again.");
                return Ok(None);
            }
            Err(_) => {
                info!(
                    "Database error, deleting row from SQLite cache and fetching translation data again."
                );
                sqlx::query(&delete_query)
                    .bind(script.text())
                    .execute(pool)
                    .await?;
                return Ok(None);
            }
        };

        let data_str: String = row.try_get("data")?;

        let data = match script {
            Script::Chinese(_) => match serde_json::from_str(&data_str) {
                Ok(v) => TranslationData::ToEnglish(v),
                Err(_) => {
                    sqlx::query(&delete_query)
                        .bind(script.text())
                        .execute(pool)
                        .await?;
                    return Ok(None);
                }
            },
            Script::English(_) => match serde_json::from_str(&data_str) {
                Ok(v) => TranslationData::ToChinese(v),
                Err(_) => {
                    sqlx::query(&delete_query)
                        .bind(script.text())
                        .execute(pool)
                        .await?;
                    return Ok(None);
                }
            },
        };

        Ok(Some(FetchedResult {
            data,
            is_cached: true,
        }))
    }

    async fn fetch_and_parse(&self, script: &Script) -> Result<TranslationData, Error> {
        let html = self.fetch_text_html(script.text()).await?;

        let binding = Html::parse_document(&html);
        let dict_page = DictPage::new(
            binding
                .select(&selectors::BODY_SELECTOR)
                .next()
                .ok_or(Error::Parse("no .search_result-dict found".into()))?,
        );

        let input = script.text();

        match script {
            Script::Chinese(_) => match dict_page.to_english(input) {
                Ok(t) => Ok(TranslationData::ToEnglish(t)),
                Err(Error::NoTranslationResults) => {
                    Ok(TranslationData::NotFound(dict_page.not_found()?))
                }
                Err(e) => Err(e),
            },
            Script::English(_) => match dict_page.to_chinese(input) {
                Ok(t) => Ok(TranslationData::ToChinese(t)),
                Err(Error::NoTranslationResults) => {
                    Ok(TranslationData::NotFound(dict_page.not_found()?))
                }
                Err(e) => Err(e),
            },
        }
    }

    async fn write_cache(
        &self,
        pool: &SqlitePool,
        script: &Script,
        data: &str,
    ) -> Result<(), Error> {
        let query = match script {
            Script::Chinese(_) => {
                "INSERT OR REPLACE INTO to_english_results (text, data) VALUES (?, ?)"
            }
            Script::English(_) => {
                "INSERT OR REPLACE INTO to_chinese_results (text, data) VALUES (?, ?)"
            }
        };

        sqlx::query(query)
            .bind(script.text())
            .bind(data)
            .execute(pool)
            .await?;

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Example, Meaning, Pronunciation, ToChinese};
    use mockito::{Matcher, Server};

    #[test]
    fn test_script_classify_chinese() {
        assert_eq!(
            Script::classify("你好").unwrap(),
            Script::Chinese("你好".to_owned())
        );
    }

    #[test]
    fn test_script_classify_english() {
        assert_eq!(
            Script::classify("hello").unwrap(),
            Script::English("hello".to_owned())
        );
    }

    #[test]
    fn test_script_classify_mixed() {
        assert_eq!(
            Script::classify("hello你好").unwrap(),
            Script::Chinese("hello你好".to_owned())
        );
    }

    #[test]
    fn test_script_classify_empty() {
        Script::classify("").unwrap_err();
    }

    #[test]
    fn test_script_text() {
        assert_eq!(Script::classify("hello").unwrap().text(), "hello");
        assert_eq!(Script::classify("你好").unwrap().text(), "你好");
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
