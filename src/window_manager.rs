use std::collections::HashMap;

use iced::{Element, window::Id};

use tracing::warn;

use crate::{ChildEvent, Event, agent::agent::AgentWindow};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowKind {
    Agent,
}

pub enum SgWindow {
    Agent(AgentWindow),
    Settings,
}

impl SgWindow {
    fn view(&self) -> Element<'_, Event> {
        match self {
            SgWindow::Agent(agent) => agent.view(),
            SgWindow::Settings => todo!(),
        }
    }

    fn update(&mut self, event: ChildEvent) {
        match self {
            SgWindow::Agent(agent) => agent.update(event),
            SgWindow::Settings => todo!(),
        }
    }

    pub fn sg_window_from_kind(id: Id, kind: WindowKind) -> SgWindow {
        match kind {
            WindowKind::Agent => SgWindow::Agent(AgentWindow::new(id)),
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
