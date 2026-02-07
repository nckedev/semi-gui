use std::collections::HashMap;

use iced::{
    Element,
    Length::Fill,
    widget::{button, text_editor::Content},
    window::Id,
};
use tracing::warn;

use crate::{ChildEvent, Event};

#[derive(Debug, Clone, Copy)]
pub enum WindowKind {
    Ai,
}

pub enum SgWindow {
    Ai(AiWindow),
    Settings,
}

impl SgWindow {
    fn view(&self) -> Element<'_, Event> {
        match self {
            SgWindow::Ai(ai_window) => ai_window.view(),
            SgWindow::Settings => todo!(),
        }
    }

    fn update(&mut self, event: ChildEvent) {}

    pub fn sg_window_from_kind(kind: WindowKind) -> SgWindow {
        match kind {
            WindowKind::Ai => SgWindow::Ai(AiWindow::default()),
        }
    }
}

#[derive(Default)]
pub struct WindowManager {
    windows: HashMap<Id, SgWindow>,
}

impl WindowManager {
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

    pub fn insert(&mut self, id: Id, window: SgWindow) {
        self.windows.insert(id, window);
    }

    pub fn remove(&mut self, id: &Id) {
        self.windows.remove(id);
    }
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

impl AiWindow {
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
