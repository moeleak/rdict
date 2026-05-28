use crate::Message;
use iced::{Element, Font, font, widget::text};

pub fn title(input_text: &str) -> Element<'_, Message> {
    text(input_text)
        .size(40)
        .font(Font {
            weight: font::Weight::ExtraBold,
            ..Font::default()
        })
        .into()
}
