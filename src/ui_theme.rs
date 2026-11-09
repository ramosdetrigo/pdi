use iced::{Color, widget::button};

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
