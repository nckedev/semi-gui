use std::time::Instant;

use iced::{Element, Task, widget::text, window::Id};

use crate::{ChildEvent, Event};

pub struct Settings {
    id: Id,
}

impl Settings {
    pub fn new(id: Id) -> Self {
        Self { id }
    }

    pub fn view(&self) -> Element<'_, Event> {
        text("test").into()
    }
    pub fn update(&mut self, event: ChildEvent) -> iced::Task<Event> {
        Task::none()
    }

    pub fn animate(&mut self, now: Instant) {}

    pub fn is_aninating(&self) -> bool {
        false
    }
}
