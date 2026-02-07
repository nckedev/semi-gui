use std::{collections::HashMap, process::Child};

use iced::{
    Element,
    Length::Fill,
    widget::{button, container, text, text_editor::Content},
    window::Id,
};
use tracing::warn;

use crate::{ChildEvent, Event};

#[derive(Debug, Clone, Copy)]
pub enum WindowKind {
    Ai,
}

#[derive(Default)]
pub struct WindowManager<M> {
    windows: HashMap<Id, Box<dyn ChildWindow<M>>>,
}

impl<M> WindowManager<M> {
    pub fn view(&self, id: Id) -> Option<iced::Element<'_, Event>> {
        self.windows.get(&id).map(|w| w.view())
    }

    pub fn update(&mut self, id: Id, event: ChildEvent) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.update(event)
        } else {
            warn!("id did not exist in windowmanager");
        }
    }

    pub fn insert<W>(&mut self, id: Id, window: W)
    where
        W: ChildWindow<M> + 'static,
    {
        self.windows.insert(id, Box::new(window));
    }

    pub fn remove(&mut self, id: &Id) {
        self.windows.remove(id);
    }
}

pub trait ChildWindow<M> {
    fn view(&self) -> iced::Element<'_, Event>;
    fn update(&mut self, event: ChildEvent);
}

pub struct AiWindow {
    chat_content: Content,
}

impl Default for AiWindow {
    fn default() -> Self {
        Self {
            chat_content: Content::with_text("234"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AiEvent {
    Todo,
}

impl From<AiEvent> for Event {
    fn from(value: AiEvent) -> Self {
        todo!()
    }
}

impl<M> ChildWindow<M> for AiWindow {
    fn view(&self) -> iced::Element<'_, Event> {
        iced::widget::column!(
            iced::widget::text_editor(&self.chat_content).height(Fill),
            button("test").on_press(Event::RequestSendNeovimCommand("!ls".to_string()))
        )
        .into()
    }

    fn update(&mut self, event: ChildEvent) {
        if let ChildEvent::Ai(event) = event {
            match event {
                AiEvent::Todo => todo!(),
            }
        }
    }
}
