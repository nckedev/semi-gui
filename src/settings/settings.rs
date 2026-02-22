use std::time::Instant;

use iced::{
    Alignment, Background, Border, Element, Length, Task, Theme,
    border::Radius,
    color,
    widget::{self, text},
    window::Id,
};
use iced_ext::IcedExt;

use crate::{ChildEvent, Event, settings::general_settings::Genral};

#[derive(Debug, Clone, Copy)]
pub enum SettingsEvent {
    PageChanged(SettingsPage),
}
impl SettingsEvent {
    pub fn into_event(self, id: Id) -> Event {
        Event::ChildEvent(id, ChildEvent::Settings(self))
    }
}

pub struct Settings {
    id: Id,
    current_page: SettingsPage,
    genral_settings: Genral,
}

impl Settings {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            current_page: SettingsPage::General,
            genral_settings: Genral::default(),
        }
    }

    pub fn view(&self) -> impl Into<Element<'_, Event>> {
        let tabs = widget::row![
            tab(
                "General",
                self.current_page,
                SettingsPage::General,
                SettingsEvent::PageChanged(SettingsPage::General).into_event(self.id)
            ),
            tab(
                "Ai",
                self.current_page,
                SettingsPage::Ai,
                SettingsEvent::PageChanged(SettingsPage::Ai).into_event(self.id)
            ),
        ]
        .align_y(Alignment::End);

        let page = match self.current_page {
            SettingsPage::General => self.genral_settings.view(),
            SettingsPage::Ai => text("ai").into(),
        };

        let action = widget::row![
            widget::space::horizontal(),
            widget::button("Save"),
            widget::button("Close").on_press(Event::CloseWindowRequested(self.id)),
        ]
        .spacing(10);

        widget::column![tabs, page, widget::space::vertical(), action].padding(10)
    }
    pub fn update(&mut self, event: ChildEvent) -> iced::Task<Event> {
        let ChildEvent::Settings(settings_event) = event else {
            return Task::none();
        };
        match settings_event {
            SettingsEvent::PageChanged(settings_page) => self.current_page = settings_page,
        }
        Task::none()
    }

    pub fn animate(&mut self, now: Instant) {}

    pub fn is_aninating(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsPage {
    General,
    Ai,
}

fn tab<'a>(
    title: &'a str,
    selected: SettingsPage,
    target: SettingsPage,
    event: Event,
) -> Element<'a, Event> {
    let is_selected = selected == target;
    let title = text(title)
        .container()
        .height(Length::Fill)
        .width(Length::Fill)
        .align_y(Alignment::End);
    widget::button(title)
        .on_press(event)
        .style(move |t: &Theme, _s| tag_style(is_selected, t))
        .height(if is_selected { 33 } else { 30 })
        .width(80)
        .into()
}

fn tag_style(selected: bool, theme: &Theme) -> widget::button::Style {
    if selected {
        iced::widget::button::Style {
            background: Some(Background::Color(theme.palette().primary)),
            border: Border {
                radius: Radius::new(0).top_right(8).top_left(8),
                ..Default::default()
            },
            ..Default::default()
        }
    } else {
        iced::widget::button::Style {
            background: Some(Background::Color(
                theme.extended_palette().background.weak.color,
            )),
            border: Border {
                radius: Radius::new(0).top_right(8).top_left(8),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
