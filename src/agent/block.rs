use iced::{
    Background, Border, Element, Renderer, Theme,
    border::Radius,
    theme::palette,
    widget::{
        self, container,
        markdown::{self},
        space, text,
    },
    window::Id,
};
use iced_ext::IcedExt;

use crate::{Event, agent::agent::AgentEvent, animated::Animated, components};

pub enum Block {
    Prompt(String),
    Answer,
    Code,
    Markdown(Vec<markdown::Item>),
    Error(String),
}

impl Block {
    pub fn view(&self, id: Id, opacity: Animated) -> Element<'_, Event> {
        match self {
            Block::Prompt(s) => prompt_view(s, id, opacity),
            Block::Answer => todo!(),
            Block::Code => todo!(),
            Block::Markdown(md) => self.markdown_view(md),
            Block::Error(err) => self.error_view(err),
        }
    }

    fn markdown_view<'a>(
        &'a self,
        md: &'a Vec<markdown::Item>,
    ) -> iced_futures::core::Element<'a, Event, Theme, Renderer> {
        let settings = markdown::Settings {
            text_size: 14.into(),
            h1_size: 14.into(),
            h2_size: 14.into(),
            h3_size: 14.into(),
            h4_size: 14.into(),
            h5_size: 14.into(),
            h6_size: 14.into(),
            code_size: 14.into(),
            spacing: 14.into(),
            style: markdown::Style::from_palette(Theme::TokyoNightLight.palette()),
        };
        // widget::markdown(
        //     md.iter(),
        //     settings
        // )
        // .map(|s| Event::Noop)
        //
        markdown::view(md, settings).map(move |s| Event::Noop)
    }

    fn error_view<'a>(&'a self, err: &'a str) -> Element<'a, Event> {
        text(err)
            .container()
            .style(|t: &Theme| container::Style {
                background: Some(Background::Color(t.palette().danger)),
                ..Default::default()
            })
            .into()
    }
}

fn prompt_view<'a>(s: &'a str, id: Id, opacity: Animated) -> Element<'a, Event> {
    const ICON_SIZE: u32 = 15;
    const TP_DELAY: u64 = 500;
    widget::column![
        widget::text(s)
            .container()
            .padding(10)
            .style(|t: &Theme| container::Style {
                border: Border {
                    radius: Radius::new(7.),
                    ..Default::default()
                },
                background: Some(Background::Color(
                    t.extended_palette().background.weakest.color,
                )),
                ..Default::default()
            }),
        container(space::horizontal()).height(20)
    ]
    .mouse_area()
    .on_enter(AgentEvent::PropmtBarShown.into_event(id))
    .on_exit(AgentEvent::PromptbarHidden.into_event(id))
    .hover(widget::column![
        space::vertical(),
        widget::row![
            space::Space::new().width(5),
            components::IconButton::new("assets/icons/clipboard.svg")
                .size(ICON_SIZE)
                .on_press(Event::Noop)
                .tooltip("Copy", TP_DELAY)
                .with_opacity(opacity.value())
                .view(),
            space::Space::new().width(5),
            components::IconButton::new("assets/icons/arrow-fork.svg")
                .size(ICON_SIZE)
                .on_press(Event::Noop)
                .tooltip("Fork", TP_DELAY)
                .view(),
            space::Space::new().width(5),
            components::IconButton::new("assets/icons/trash.svg")
                .size(ICON_SIZE)
                .hover_color(|t: &Theme| t.palette().danger)
                .on_press(Event::Noop)
                .tooltip("Delete", TP_DELAY)
                .view(),
            space::Space::new().width(2),
            space::horizontal(),
            widget::text("date").style(|t: &Theme| text::Style {
                color: Some(palette::lighten(t.palette().text, 0.35))
            }),
            space::Space::new().width(5),
        ]
        .height(20)
    ])
}
