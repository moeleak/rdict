use crate::parse::{NotFound, ToChinese, ToEnglish, TranslationData};
use owo_colors::OwoColorize;
use std::fmt::Write;

#[must_use]
pub trait Render {
    fn render_colored(&self) -> String;
    fn render_plain(&self) -> String;
}

impl Render for ToChinese {
    fn render_colored(&self) -> String {
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

    fn render_plain(&self) -> String {
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

impl Render for ToEnglish {
    fn render_colored(&self) -> String {
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

    fn render_plain(&self) -> String {
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

impl Render for NotFound {
    fn render_colored(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", "Did you mean:".bright_black()).unwrap();
        for suggestion in &self.suggestions {
            writeln!(output, "* {}", suggestion.green()).unwrap();
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
