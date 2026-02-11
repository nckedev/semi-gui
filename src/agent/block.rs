use std::marker::PhantomData;

use iced::{Element, widget};

pub enum Block<M> {
    Prompt(String),
    Answer,
    Code,
    __marker(PhantomData<M>),
}

impl<M> Block<M> {
    pub fn view(&self) -> Element<'_, M> {
        match self {
            Block::Prompt(s) => prompt_view(s),
            Block::Answer => todo!(),
            Block::Code => todo!(),
            Block::__marker(phantom_data) => todo!(),
        }
    }
}

fn prompt_view<M>(s: &str) -> Element<'_, M> {
    widget::text(s).into()
}
