use std::{fmt::Display, ops::Shr};

use iced::{
    Alignment, Background, Border, Color, Element, Font,
    Length::{Fill, Shrink},
    Padding, Theme, color,
    widget::{
        self, container,
        text_editor::{Action, Content},
    },
    window::Id,
};
use iced_ext::IcedExt;

use crate::{
    ChildEvent, Event, agent::block, ai::ai::Response, prompt::Prompt, window_manager::WindowKind,
};

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Todo,
    InputChanged(String),
    ResponseRecived(Response),
    InputEdit(Action),
    PressedSend,
}

impl AgentEvent {
    pub fn into_event(self, id: Id) -> Event {
        Event::ChildEvent(id, ChildEvent::Agent(self))
    }
}

pub struct AgentWindow {
    id: Id,
    chat_content: Content,
    chat: Vec<Block>,
    input: String,
    input_box: Content,
    is_busy: bool,
    mode: AgentMode,
}

impl AgentWindow {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            chat_content: Content::with_text("test"),
            chat: vec![],
            input: "".to_string(),
            input_box: Content::new(),
            is_busy: false,
            mode: AgentMode::Chat,
        }
    }

    pub fn view(&self) -> iced::Element<'_, Event> {
        let input_box = widget::text_editor(&self.input_box)
            .font(Font::MONOSPACE)
            .height(20 * (self.input_box.lines().count() as u32).clamp(3, 10))
            .max_height(100)
            .on_action(|event| AgentEvent::InputEdit(event).into_event(self.id))
            .container()
            .height(Shrink);

        // let chat = widget::text_editor(&self.chat_content).height(Fill);
        let btn = widget::button("Send").on_press_maybe(
            if !self.is_busy && !self.input_box.text().is_empty() {
                Some(Event::AskClanker(
                    self.id,
                    WindowKind::Agent,
                    Prompt::from(self.input_box.text().clone()),
                ))
            } else {
                None
            },
        );

        let bottom_row = widget::row![widget::text(self.mode.to_string())]
            .width(Fill)
            .height(20)
            .container()
            .style(|t| container::Style {
                background: Some(Background::Color(t.palette().primary)),
                ..Default::default()
            })
            .padding(Padding::new(0.).horizontal(10).bottom(5));

        widget::column![
            chat_content(self),
            iced::widget::stack![
                input_box,
                btn.container()
                    .height(Fill)
                    .width(Fill)
                    .align_y(Alignment::End)
                    .align_x(Alignment::End)
            ]
            .container()
            .padding(Padding::new(0.).horizontal(10))
            .align_y(Alignment::End),
            bottom_row
        ]
        .height(Fill)
        .into()
    }

    pub fn update(&mut self, event: ChildEvent) {
        if let ChildEvent::Agent(event) = event {
            match event {
                AgentEvent::Todo => {}
                AgentEvent::InputChanged(s) => {
                    // self.chat.push(Block { content: s.clone() });
                    self.chat_content =
                        Content::with_text(&format!("{:?}", sub_str(&s, diff(&self.input, &s))));
                    self.input = s;
                }
                AgentEvent::ResponseRecived(r) => {
                    self.is_busy = false;
                    self.chat.push(Block {
                        content: r.content.clone(),
                    })
                }
                AgentEvent::InputEdit(action) => {
                    self.input_box.perform(action);
                    // tracing::info!("{event:?}");
                    // match event {
                    //     Action::Move(motion) => {}
                    //     Action::Select(motion) => {}
                    //     Action::SelectWord => {}
                    //     Action::SelectLine => {}
                    //     Action::SelectAll => {}
                    //     Action::Edit(edit) => match edit {
                    //         widget::text_editor::Edit::Insert(chr) => self.input_box.perform(event),
                    //         widget::text_editor::Edit::Paste(_) => todo!(),
                    //         widget::text_editor::Edit::Enter => todo!(),
                    //         widget::text_editor::Edit::Indent => todo!(),
                    //         widget::text_editor::Edit::Unindent => todo!(),
                    //         widget::text_editor::Edit::Backspace => todo!(),
                    //         widget::text_editor::Edit::Delete => todo!(),
                    //     },
                    //     Action::Click(point) => {}
                    //     Action::Drag(point) => {}
                    //     Action::Scroll { lines } => {}
                    // }
                }
                AgentEvent::PressedSend => {
                    self.is_busy = true;
                    self.input_box = Content::with_text("")
                }
            };
        }
    }
}

fn contains_at(str: &str) -> Option<usize> {
    // "im editing this @stri now"
    // "im editing this @stirn now"
    if let Some(pair) = str.rsplit_once(' ') {
        if pair.1.starts_with('@') {
            return Some(1);
        }
    }
    None
}

fn diff(first: &str, second: &str) -> usize {
    let pos = first.chars().zip(second.chars()).position(|(a, b)| a != b);
    if let Some(pos) = pos {
        return pos;
    }

    second.len()
}

fn sub_str(str: &str, idx: usize) -> &str {
    debug_assert!(str.len() >= idx);
    let start = match str[..idx].rfind(char::is_whitespace) {
        Some(pos) => pos + 1,
        None => 0,
    };

    let end = match str[idx..].find(char::is_whitespace) {
        Some(pos) => idx + pos,
        None => str.len(),
    };

    &str[start..end]
}

fn chat_content(state: &AgentWindow) -> Element<'_, Event> {
    let blocks = state.chat.iter().map(|b| {
        widget::text(&b.content).width(Fill).hover(widget::row![
            widget::space::horizontal(),
            widget::text("test"),
            widget::space::Space::new().width(15)
        ])
    });
    widget::scrollable(
        widget::Column::from_iter(blocks)
            .width(Fill)
            .container()
            .padding(10)
            .width(Fill),
    )
    .height(Fill)
    .into()
}

enum AgentMode {
    Chat,
    Agent,
}

impl Display for AgentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentMode::Chat => write!(f, "Chat"),
            AgentMode::Agent => write!(f, "Agent"),
        }
    }
}

pub struct Block {
    content: String,
}

struct Keyword<'a> {
    start: usize,
    end: usize,
    kind: KeywordKind,
    raw: &'a str,
}

enum KeywordKind {
    // !
    Role,
    //@ file, selected, current(file)
    Include,
    //@@
    Lsp,
    // #
    Template,
}
