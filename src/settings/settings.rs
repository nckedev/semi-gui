use std::time::Instant;

use iced::{
    Element, Task,
    widget::{self, text},
    window::Id,
};

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
            current_page: SettingsPage::Genaral,
            genral_settings: Genral::default(),
        }
    }

    pub fn view(&self) -> impl Into<Element<'_, Event>> {
        let title = widget::text("SETTINGS");

        let tabs = widget::row![
            widget::button("General")
                .on_press(SettingsEvent::PageChanged(SettingsPage::Genaral).into_event(self.id)),
            widget::button("Ai")
                .on_press(SettingsEvent::PageChanged(SettingsPage::Ai).into_event(self.id)),
        ]
        .spacing(10);

        let page = match self.current_page {
            SettingsPage::Genaral => self.genral_settings.view(),
            SettingsPage::Ai => text("ai").into(),
        };

        let action = widget::row![
            widget::space::horizontal(),
            widget::button("Save"),
            widget::button("Close").on_press(Event::CloseWindowRequested(self.id)),
        ]
        .spacing(10);

        widget::column![title, tabs, page, widget::space::vertical(), action].padding(10)
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

#[derive(Debug, Clone, Copy)]
pub enum SettingsPage {
    Genaral,
    Ai,
}
