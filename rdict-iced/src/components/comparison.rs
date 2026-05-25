use crate::Message;
use iced::{
    Element, Font, font,
    widget::{column, text},
};

pub fn comparison<'a>(first: &'a str, second: &'a str) -> Element<'a, Message> {
    column![
        text(first).font(Font {
            weight: font::Weight::Medium,
            ..Font::default()
        }),
        text(second).size(14).style(text::secondary),
    ]
    .spacing(5)
    .into()
}
