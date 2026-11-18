// Mine — Windows XP classic visual theme

use iced::border::Radius;
use iced::widget::{button, container, text, text_input};
use iced::{Background, Border, Color, Theme};
use iced::theme::Palette;

pub const XP_FACE: Color = Color::from_rgb(0.831, 0.816, 0.784); // #D4D0C8
pub const XP_WHITE: Color = Color::from_rgb(1.0, 1.0, 1.0);
pub const XP_TITLE: Color = Color::from_rgb(0.039, 0.141, 0.416); // #0A246A
pub const XP_TITLE_LIGHT: Color = Color::from_rgb(0.651, 0.792, 0.941); // #A6CAF0
pub const XP_SHADOW: Color = Color::from_rgb(0.502, 0.502, 0.502);
pub const XP_DARK_SHADOW: Color = Color::from_rgb(0.251, 0.251, 0.251);
pub const XP_TEXT: Color = Color::from_rgb(0.0, 0.0, 0.0);
pub const XP_DISABLED: Color = Color::from_rgb(0.502, 0.502, 0.502);
pub const XP_SUCCESS: Color = Color::from_rgb(0.0, 0.502, 0.0);
pub const XP_DANGER: Color = Color::from_rgb(0.752, 0.0, 0.0);

pub fn xp_theme() -> Theme {
    Theme::custom(
        "Windows XP".to_string(),
        Palette {
            background: XP_FACE,
            text: XP_TEXT,
            primary: XP_TITLE,
            success: XP_SUCCESS,
            danger: XP_DANGER,
            warning: Color::from_rgb(0.8, 0.6, 0.0),
        },
    )
}

pub fn face_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(XP_FACE)),
        text_color: Some(XP_TEXT),
        border: Border {
            color: XP_SHADOW,
            width: 1.0,
            radius: Radius::from(0.0),
        },
        ..Default::default()
    }
}

pub fn white_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(XP_WHITE)),
        text_color: Some(XP_TEXT),
        border: Border {
            color: XP_SHADOW,
            width: 1.0,
            radius: Radius::from(0.0),
        },
        ..Default::default()
    }
}

pub fn title_bar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(XP_TITLE)),
        text_color: Some(XP_WHITE),
        border: Border {
            color: XP_TITLE,
            width: 0.0,
            radius: Radius::from(0.0),
        },
        ..Default::default()
    }
}

pub fn inset_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(XP_WHITE)),
        text_color: Some(XP_TEXT),
        border: Border {
            color: XP_DARK_SHADOW,
            width: 1.0,
            radius: Radius::from(0.0),
        },
        ..Default::default()
    }
}

pub fn xp_button(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Pressed => Color::from_rgb(0.78, 0.76, 0.73),
        button::Status::Hovered => Color::from_rgb(0.90, 0.89, 0.86),
        button::Status::Disabled => XP_FACE,
        button::Status::Active => XP_FACE,
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: if matches!(status, button::Status::Disabled) {
            XP_DISABLED
        } else {
            XP_TEXT
        },
        border: Border {
            color: match status {
                button::Status::Pressed => XP_DARK_SHADOW,
                _ => XP_SHADOW,
            },
            width: 1.0,
            radius: Radius::from(0.0),
        },
        ..Default::default()
    }
}

pub fn xp_tab(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let mut style = xp_button(theme, status);
        if active {
            style.background = Some(Background::Color(XP_WHITE));
            style.border.color = XP_TITLE;
            style.border.width = 2.0;
        }
        style
    }
}

pub fn xp_input(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border = match status {
        text_input::Status::Focused { .. } => Border {
            color: XP_TITLE,
            width: 2.0,
            radius: Radius::from(0.0),
        },
        _ => Border {
            color: XP_DARK_SHADOW,
            width: 1.0,
            radius: Radius::from(0.0),
        },
    };

    text_input::Style {
        background: Background::Color(XP_WHITE),
        border,
        icon: XP_TEXT,
        placeholder: XP_DISABLED,
        value: XP_TEXT,
        selection: XP_TITLE_LIGHT,
    }
}

pub fn label_text<'a>(content: impl Into<String>) -> text::Text<'a> {
    text(content.into()).size(13).color(XP_TEXT)
}

pub fn title_text<'a>(content: impl Into<String>) -> text::Text<'a> {
    text(content.into()).size(14).color(XP_WHITE)
}

pub fn section_header<'a>(content: impl Into<String>) -> text::Text<'a> {
    text(content.into()).size(15).color(XP_TITLE)
}
