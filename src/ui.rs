use iced::{Color, Theme, widget::{Rule, button, rule}};

use crate::Message;

pub fn cbutton<'a>(
    content: impl Into<iced::Element<'a, Message>>,
) -> iced::widget::Button<'a, Message> {
    button(content).style(|theme: &iced::Theme, status| button::Style {
        background: match status {
            iced::widget::button::Status::Hovered => Some(iced::Background::Color(
                Color::from_rgb(0.282, 0.267, 0.247),
            )),
            iced::widget::button::Status::Pressed => Some(iced::Background::Color(
                Color::from_rgb(0.180, 0.169, 0.161),
            )),
            _ => Some(iced::Background::Color(theme.palette().primary)),
        },
        text_color: theme.palette().text,
        border: iced::Border {
            radius: 0.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
}

pub fn hseparator<'a>() -> Rule<'a, Theme> {
    rule::horizontal(2).style(|_theme| rule::Style {
        color: Color::from_rgb(0.545, 0.518, 0.467),
        radius: iced::border::Radius::new(0.0),
        fill_mode: rule::FillMode::Full,
        snap: true,
    })
}

pub fn vseparator<'a>() -> Rule<'a, Theme> {
    rule::vertical(2).style(|_theme| rule::Style {
        color: Color::from_rgb(0.545, 0.518, 0.467),
        radius: iced::border::Radius::new(0.0),
        fill_mode: rule::FillMode::Full,
        snap: true,
    })
}
