use crate::Message;
use iced::Font;
use iced::font::{Family, Weight};
use iced::widget::text;
use iced_material as material;

pub fn title(input_text: &str) -> material::Element<'_, Message> {
    text(input_text)
        .size(40)
        .style(material::text::surface)
        .font(title_font(input_text))
        .into()
}

fn title_font(input_text: &str) -> Font {
    if cfg!(target_os = "android") && material::fonts::contains_cjk(input_text) {
        return Font {
            weight: Weight::Bold,
            ..Font::default()
        };
    }

    let cjk_family = if contains_japanese_kana(input_text) {
        Some(system_japanese_font())
    } else if contains_hangul(input_text) {
        Some(system_korean_font())
    } else if material::fonts::contains_cjk(input_text) {
        Some(system_chinese_font())
    } else {
        None
    };

    cjk_family.map_or(material::fonts::ROBOTO_BOLD, |family| Font {
        family: Family::Name(family),
        weight: Weight::Bold,
        ..Font::default()
    })
}

fn contains_japanese_kana(input_text: &str) -> bool {
    input_text
        .chars()
        .any(|c| matches!(c, '\u{3040}'..='\u{30FF}' | '\u{31F0}'..='\u{31FF}'))
}

fn contains_hangul(input_text: &str) -> bool {
    input_text.chars().any(|c| {
        matches!(
            c,
            '\u{1100}'..='\u{11FF}'
                | '\u{3130}'..='\u{318F}'
                | '\u{A960}'..='\u{A97F}'
                | '\u{AC00}'..='\u{D7AF}'
                | '\u{D7B0}'..='\u{D7FF}'
        )
    })
}

fn system_chinese_font() -> &'static str {
    if cfg!(target_os = "macos") {
        "PingFang SC"
    } else if cfg!(target_os = "windows") {
        "Microsoft YaHei"
    } else {
        "Noto Sans CJK SC"
    }
}

fn system_japanese_font() -> &'static str {
    if cfg!(target_os = "macos") {
        "Hiragino Sans"
    } else if cfg!(target_os = "windows") {
        "Yu Gothic"
    } else {
        "Noto Sans CJK JP"
    }
}

fn system_korean_font() -> &'static str {
    if cfg!(target_os = "macos") {
        "Apple SD Gothic Neo"
    } else if cfg!(target_os = "windows") {
        "Malgun Gothic"
    } else {
        "Noto Sans CJK KR"
    }
}
