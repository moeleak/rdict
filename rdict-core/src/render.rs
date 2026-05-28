// TODO: This is shared by rdict_cli and rdict_telegram.
//       I need to find a way to share this between them but without putting this inside rdict_core.
//
// TODO: This really should be moved to rdict_cli...
//       but impl doesn't work for external data types

mod en;
mod fr;
mod ja;
mod ko;

use crate::parse::NotFound;
use owo_colors::OwoColorize;
use std::fmt::Write;

pub mod colors {
    use owo_colors::Style;

    pub const PRIMARY: Style = Style::new().green();

    pub const SECONDARY: Style = Style::new().magenta();

    pub const MUTED: Style = Style::new().bright_black();
}

#[must_use]
pub trait Render {
    // TODO: It's kind of duplicate
    fn render_colored(&self) -> String;
    fn render_plain(&self) -> String;
}

impl Render for NotFound {
    fn render_colored(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", "Did you mean:".style(colors::MUTED)).unwrap();
        for suggestion in &self.suggestions {
            writeln!(output, "* {}", suggestion.style(colors::PRIMARY)).unwrap();
        }

        output.trim_end().to_string()
    }

    fn render_plain(&self) -> String {
        let mut output = String::new();

        writeln!(output, "Did you mean:").unwrap();
        for suggestion in &self.suggestions {
            writeln!(output, "* {suggestion}").unwrap();
        }

        output.trim_end().to_string()
    }
}
