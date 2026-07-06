pub mod en;
pub mod fr;
pub mod ja;
pub mod ko;

use crate::Message;
use iced::{Length, Padding, alignment, widget::container};
use iced_material as material;
use material::widget::button;
use rdict_core::model::Voice;

pub(crate) const PRONUNCIATION_HEIGHT: f32 = 40.0;
const PRONUNCIATION_PADDING_X: u16 = 12;

pub(crate) fn pronunciation<'a>(
    value: String,
    voice: Option<&Voice>,
) -> material::Element<'a, Message> {
    let label = format!("[{value}]");

    if let Some(voice) = voice {
        button::Button::new(
            container(material::text::body_large(label))
                .height(Length::Fixed(PRONUNCIATION_HEIGHT))
                .padding([0, PRONUNCIATION_PADDING_X])
                .align_y(alignment::Vertical::Center),
        )
        .width(Length::Shrink)
        .height(Length::Fixed(PRONUNCIATION_HEIGHT))
        .padding(Padding::ZERO)
        .style(material::style::button::text)
        .on_press(Message::PlayVoice(voice.clone()))
        .into()
    } else {
        container(material::text::body_large(label).style(material::text::surface_variant))
            .height(Length::Fixed(PRONUNCIATION_HEIGHT))
            .align_y(alignment::Vertical::Center)
            .into()
    }
}
