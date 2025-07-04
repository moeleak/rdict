// TODO:
// * Better font rendering.
// * Don't use plain text to show result.
// * App icon.
// * Styling & auto dark mode.
// * Polish the UI
//   e.g. tooltips, colors, shadows, popups.
// * Add more features like history, favorites, etc.
// * Add tests.

use anyhow::{Context, Result};
use directories_next::ProjectDirs;
use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::window::Settings;
use iced::{Element, Length, Size, Task, Theme};
use rdict_core::parse::TranslationData;
use rdict_core::rdict::{self, Rdict};

#[derive(Default)]
struct State {
    text_input_content: String,
    translation_result: String,
    client: Option<Rdict>,
}

#[derive(Debug, Clone)]
enum Message {
    ContentChanged(String),
    Submit,
    TranslationResult(String),
    TranslationError(String),
}

fn main() -> Result<()> {
    iced::application(State::default, update, view)
        .title("Rdict")
        .window(Settings {
            size: Size::new(400.0, 600.0),
            resizable: true,
            decorations: true,
            ..Settings::default()
        })
        .theme(|_| Theme::GruvboxDark)
        .run()?;

    Ok(())
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::ContentChanged(content) => {
            state.text_input_content = content;
            Task::none()
        }
        Message::Submit => {
            state.translation_result = "Translating...".to_string();
            let text_input_content = state.text_input_content.clone();
            let client = state.client.clone();
            Task::perform(
                async move { result(client, text_input_content).await },
                |res| match res {
                    Ok(msg) => Message::TranslationResult(msg),
                    Err(e) => Message::TranslationError(e.to_string()),
                },
            )
        }
        Message::TranslationResult(msg) => {
            state.translation_result = msg;
            Task::none()
        }
        Message::TranslationError(err) => {
            state.translation_result = format!("Error: {err}");
            Task::none()
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let mut col = column![
        row![
            text_input("Type something here...", &state.text_input_content)
                .on_input(Message::ContentChanged)
                .on_submit(Message::Submit),
            button("Translate").on_press(Message::Submit),
        ]
        .spacing(10),
        // Expanded result text
        scrollable(
            text(&state.translation_result)
                .size(16)
                .shaping(iced::widget::text::Shaping::Advanced)
        )
        .width(Length::Fill)
        .height(Length::Fill),
    ]
    .padding(10)
    .spacing(10);

    // Conditionally push the debug footer
    if cfg!(debug_assertions) {
        col = col.push(text("rdict_iced dev").align_x(iced::alignment::Horizontal::Center));
    }

    col.into()
}

async fn result(client: Option<Rdict>, text_input_content: String) -> Result<String> {
    let client = if let Some(c) = client {
        c
    } else {
        let cache_db_path = ProjectDirs::from("dev", "ny4", "rdict")
            .map(|proj_dirs| proj_dirs.cache_dir().join("cache.db"));
        Rdict::new("https://m.youdao.com", cache_db_path).await?
    };

    let result = client
        .get_results(&text_input_content)
        .await
        .context("Failed to get translation results")?;

    match result.data {
        TranslationData::ToChinese(tc) => {
            rdict::render_chinese_plain(&tc).context("Failed to render Chinese translation")
        }
        TranslationData::ToEnglish(te) => {
            rdict::render_english_plain(&te).context("Failed to render English translation")
        }
    }
}
