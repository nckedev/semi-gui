use std::{
    collections::{HashMap, hash_map},
    time::Instant,
};

use iced::{Element, Task, window::Id};

use tracing::warn;

use crate::{
    ChildEvent, Event,
    agent::agent::{AgentEvent, AgentWindow},
    settings::settings::{self, Settings},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowKind {
    Agent,
    Settings,
}

impl WindowKind {
    /// Creates a user facing error for the relevant page as an event
    pub fn create_user_error(&self, str: String, id: Id) -> Event {
        match self {
            WindowKind::Agent => AgentEvent::ErrorHappend(str).into_event(id),
            _ => todo!("not implemented yet"),
        }
    }
}

pub enum SgWindow {
    Agent(AgentWindow),
    Settings(Settings),
}

impl SgWindow {
    fn view(&self) -> Element<'_, Event> {
        match self {
            SgWindow::Agent(agent) => agent.view(),
            SgWindow::Settings(settings) => settings.view(),
        }
    }

    fn update(&mut self, event: ChildEvent) -> Task<Event> {
        match self {
            SgWindow::Agent(agent) => agent.update(event),
            SgWindow::Settings(settings) => settings.update(event),
        }
    }

    fn animate(&mut self, now: Instant) {
        match self {
            SgWindow::Agent(agent) => agent.animate(now),
            SgWindow::Settings(settings) => settings.animate(now),
        }
    }

    fn is_animating(&self) -> bool {
        match self {
            SgWindow::Agent(agent) => agent.is_animating(),
            SgWindow::Settings(settings) => settings.is_aninating(),
        }
    }

    pub fn sg_window_from_kind(id: Id, kind: WindowKind) -> SgWindow {
        match kind {
            WindowKind::Agent => SgWindow::Agent(AgentWindow::new(id)),
            WindowKind::Settings => SgWindow::Settings(settings::Settings::new(id)),
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

    pub fn update(&mut self, id: Id, event: ChildEvent) -> Task<Event> {
        if let Some(w) = self.windows.get_mut(&id) {
            w.update(event)
        } else {
            warn!("id did not exist in windowmanager");
            Task::none()
        }
    }

    pub fn animate(&mut self, now: Instant) {
        for w in self.windows.values_mut() {
            w.animate(now);
        }
    }

    pub fn is_animating(&self) -> bool {
        self.windows.iter().all(|(_, w)| w.is_animating())
    }

    pub fn insert(&mut self, id: Id, window: SgWindow) {
        self.windows.insert(id, window);
    }

    pub fn find_first(&self, kind: WindowKind) -> Option<Id> {
        for (id, window) in &self.windows {
            match (kind, window) {
                (WindowKind::Settings, SgWindow::Settings(..)) => return Some(*id),
                (WindowKind::Agent, SgWindow::Agent(..)) => return Some(*id),
                _ => {}
            }
        }
        None
    }

    pub fn remove(&mut self, id: &Id) {
        self.windows.remove(id);
    }

    pub fn iter(&self) -> hash_map::Iter<'_, Id, SgWindow> {
        self.windows.iter()
    }

    pub fn is_agent(&self, id: Id) -> bool {
        if let Some(wnd) = self.windows.get(&id)
            && matches!(wnd, SgWindow::Agent(..))
        {
            return true;
        }
        false
    }
}
