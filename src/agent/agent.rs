use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    time::Instant,
};

use iced::{
    Alignment, Background, Border, Color, Element, Font,
    Length::{Fill, Shrink},
    Padding, Task, Theme,
    border::Radius,
    keyboard::Key,
    theme::palette,
    widget::{
        self, canvas, container, markdown, text,
        text_editor::{Action, Binding, Content},
    },
    window::Id,
};
use iced_ext::IcedExt;

use crate::{
    ChildEvent, Event,
    agent::block::Block,
    ai::ai::Response,
    animated::Animated,
    components::{self, spinner::Spinner},
    prompt::Prompt,
    style,
    window_manager::WindowKind,
};

struct Animations {
    sidebar_width: Animated<f32>,
    session_opacity: Animated<f32>,
    icons_opacity: Animated,
}

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Todo,
    InputChanged(String),
    ResponseRecived(Response),
    InputEdit(Action),
    PressedSend,
    SideBarCollapse,
    PropmtBarShown,
    PromptbarHidden,
    ErrorHappend(String),
}

impl AgentEvent {
    pub fn into_event(self, id: Id) -> Event {
        Event::ChildEvent(id, ChildEvent::Agent(self))
    }
}

const SIDEBAR_OPEN_WIDTH: f32 = 200.;
const SIDEBAR_CLOSED_WIDHT: f32 = 46.;
const COLLAPSE_BTN_SIZE: u32 = 30;

pub struct AgentWindow {
    id: Id,
    chat_content: Content,
    chat: Vec<Block>,
    input: String,
    input_box: Content,
    is_busy: bool,
    mode: AgentMode,
    prev_id: Option<String>,
    sidebar_collapsed: bool,
    session: HashMap<String, String>,
    selected_session: String,
    md: Vec<markdown::Item>,
    animations: Animations,
    spinner: Spinner,
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
            prev_id: None,
            session: HashMap::new(),
            selected_session: String::from(""),
            sidebar_collapsed: false,
            md: vec![],
            animations: Animations {
                sidebar_width: Animated::new(SIDEBAR_OPEN_WIDTH)
                    .animate_to(SIDEBAR_CLOSED_WIDHT)
                    .with_duration(200)
                    .build(),
                session_opacity: Animated::new(1.),
                icons_opacity: Animated::new(0.)
                    .with_delay(1000)
                    .with_duration(500)
                    .animate_to(1.)
                    .build(),
            },
            spinner: Spinner::new(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Event> {
        let input_box = widget::text_editor(&self.input_box)
            .font(Font::MONOSPACE)
            .height(20 * (self.input_box.lines().count() as u32).clamp(3, 10))
            .max_height(100)
            .key_binding(|k| {
                // let all command-<chr> throuh except cmd-c,v and x
                if k.modifiers.command()
                    && k.key != Key::Character("c".into())
                    && k.key != Key::Character("v".into())
                    && k.key != Key::Character("x".into())
                {
                    None
                } else {
                    Binding::from_key_press(k)
                }
            })
            .on_action(|event| AgentEvent::InputEdit(event).into_event(self.id))
            .container()
            .height(Shrink);

        // let chat = widget::text_editor(&self.chat_content).height(Fill);
        // let btn = widget::button("Send").on_press_maybe(
        //     if !self.is_busy && !self.input_box.text().is_empty() {
        //         Some(Event::AskClanker(
        //             self.id,
        //             WindowKind::Agent,
        //             self.prev_id.clone(),
        //             Prompt::from(self.input_box.text().clone()),
        //         ))
        //     } else {
        //         None
        //     },
        // );

        let btn2 = components::IconButton::new("assets/icons/send.svg")
            .size(38)
            .on_press_maybe(if !self.is_busy && !self.input_box.text().is_empty() {
                Some(AgentEvent::PressedSend.into_event(self.id))
                // Some(Event::AskClanker(
                //     self.id,
                //     WindowKind::Agent,
                //     self.prev_id.clone(),
                //     Prompt::from(self.input_box.text().clone()),
                // ))
            } else {
                None
            })
            .view();

        let path = if self.sidebar_collapsed {
            "assets/icons/square-rounded-arrow-right.svg"
        } else {
            "assets/icons/square-rounded-arrow-left.svg"
        };

        let sessions = widget::column![
            // sessions titlebar
            widget::row![
                if self.animations.sidebar_width.value() > SIDEBAR_OPEN_WIDTH * 0.5 {
                    Some(widget::text("Sessions").style(|_| text::Style {
                        color: Some(Color::from_rgba(
                            0.,
                            0.,
                            0.,
                            self.animations.session_opacity.value(),
                        )),
                    }))
                } else {
                    None
                },
                widget::space::horizontal(),
                components::IconButton::new(path)
                    .size(COLLAPSE_BTN_SIZE)
                    .on_press(AgentEvent::SideBarCollapse.into_event(self.id))
                    .view()
            ]
            .align_y(Alignment::Center),
            widget::column![
                // session card
                widget::column![
                    widget::text("Title"),
                    widget::text("Test this is some generated content.. ")
                        .size(14.)
                        .style(|t: &Theme| {
                            text::Style {
                                color: Some(palette::lighten(
                                    t.extended_palette().background.base.text,
                                    0.35,
                                )),
                            }
                        }),
                ]
                .container()
                .width(Fill)
                .padding(5)
                .style(|t| container::Style {
                    background: Some(Background::Color(
                        t.extended_palette().background.base.color
                    )),
                    border: Border {
                        radius: Radius::new(5.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .show_if(
                    !self.sidebar_collapsed
                        && self.animations.sidebar_width.value() > (SIDEBAR_OPEN_WIDTH * 0.9)
                ),
                widget::space::vertical(),
                widget::row![
                    components::IconButton::new("assets/icons/settings.svg")
                        .with_text_maybe(if !self.sidebar_collapsed {
                            Some("Settings")
                        } else {
                            None
                        })
                        .on_press(Event::OpenWindowRequested(WindowKind::Settings)),
                ]
            ]
        ]
        .container()
        .padding(8)
        .style(|_t| container::Style {
            background: Some(Background::Color(
                _t.extended_palette().background.weakest.color,
            )),
            ..Default::default()
        })
        .height(Fill)
        .width(self.animations.sidebar_width.value());

        let spinner = canvas(&self.spinner).height(50).width(Fill);

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
            widget::row![widget::space::horizontal(), widget::text("test")]
                .height(30)
                .align_y(Alignment::Center)
                .padding(Padding::new(0.).right(10.)),
            widget::row![
                widget::row![sessions],
                widget::column![
                    chat_content(self),
                    spinner,
                    iced::widget::stack![
                        input_box,
                        btn2.container()
                            .padding(5)
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
            ]
            .height(Fill)
        ]
        .into()
    }

    pub fn animate(&mut self, now: Instant) {
        self.animations.sidebar_width.update(now);
        self.animations.session_opacity.update(now);
        self.animations.icons_opacity.update(now);
        self.spinner.update(now);
    }

    pub fn is_animating(&self) -> bool {
        self.animations.sidebar_width.is_animating()
            || self.animations.session_opacity.is_animating()
            || self.animations.icons_opacity.is_animating()
    }

    pub fn update(&mut self, event: ChildEvent) -> Task<Event> {
        if let ChildEvent::Agent(event) = event {
            match event {
                AgentEvent::ErrorHappend(e) => {
                    self.chat.push(Block::Error(e));
                    self.is_busy = false;
                }
                AgentEvent::SideBarCollapse => {
                    self.sidebar_collapsed = !self.sidebar_collapsed;
                    if self.sidebar_collapsed {
                        self.animations.sidebar_width.start();
                        self.animations
                            .session_opacity
                            .animate_to(0.)
                            .with_duration(100)
                            .with_delay(0)
                            .start();
                    } else {
                        self.animations.sidebar_width.start_reverse();
                        self.animations
                            .session_opacity
                            .animate_to(1.)
                            .with_duration(100)
                            .with_delay(200)
                            .start();
                    }
                }
                AgentEvent::Todo => {}
                AgentEvent::InputChanged(s) => {
                    // self.chat.push(Block { content: s.clone() });
                    self.chat_content =
                        Content::with_text(&format!("{:?}", sub_str(&s, diff(&self.input, &s))));
                    self.input = s;
                }
                AgentEvent::ResponseRecived(r) => {
                    self.is_busy = false;
                    self.prev_id = Some(r.id);
                    self.chat
                        .push(Block::Markdown(markdown::parse(&r.content).collect()));
                    // self.md = markdown::parse(&r.content).collect();
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
                    self.chat.push(Block::Prompt(self.input_box.text()));
                    self.is_busy = true;

                    // add a space for easier parsing
                    let mut str = self.input_box.text();
                    str.push(' ');
                    self.input_box = Content::with_text("");

                    return Task::done(Event::AskClanker(
                        self.id,
                        WindowKind::Agent,
                        self.prev_id.clone(),
                        Prompt::from(str),
                        |r, id| AgentEvent::ResponseRecived(r).into_event(id),
                    ));
                }
                AgentEvent::PropmtBarShown => {
                    tracing::info!("starting opacity");
                    self.animations.icons_opacity.start();
                }
                AgentEvent::PromptbarHidden => {
                    tracing::info!("starting reverse");
                    self.animations.icons_opacity.start_reverse();
                }
            };
        }
        Task::none()
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
    let blocks = state
        .chat
        .iter()
        .map(|b| b.view(state.id, state.animations.icons_opacity));
    widget::Column::from_iter(blocks)
        .width(Fill)
        .padding(10)
        .scrollale()
        .style(style::main_scrollbar)
        .height(Fill)
        .into()
}

enum AgentMode {
    Chat,
    Agent,
    Diff,
}

impl Display for AgentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentMode::Chat => write!(f, "Chat"),
            AgentMode::Agent => write!(f, "Agent"),
            AgentMode::Diff => write!(f, "Diff"),
        }
    }
}

pub struct Session {
    id: u32,
    title: Option<String>,
    content: Vec<Block>,
    prev_id: String,
    pinned: bool,
    forked_from: Option<u32>,
    timestamp: u32, // TODO:
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
