use crate::Message;
use iced::{
    Element, Font, font,
    widget::{column, container, rule, text},
};

pub fn section<'a>(
    title: &'a str,
    children: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        text(title).style(text::secondary).font(Font {
            weight: font::Weight::Bold,
            ..Font::default()
        }),
        rule::horizontal(1),
        container(children)
    ]
    .spacing(10)
    .into()
}
