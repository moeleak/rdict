use crate::Message;
use iced::{
    Alignment, Element,
    widget::{row, text},
};

pub fn list_item<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    row![
        // FIXME: use proper way to render the dot
        text("•").size(20),
        content.into()
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}
