use iced::{
    Element,
    widget::{self, text},
};

use crate::Event;

#[derive(Default)]
pub struct Genral {}

impl Genral {
    pub fn view(&self) -> Element<'_, Event> {
        widget::text("general").into()
    }
}
