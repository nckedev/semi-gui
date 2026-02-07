mod commands;
mod window_manager;

use std::{fmt::Debug, fs::File};

use async_trait::async_trait;
use iced::{
    Element, Subscription, Task,
    futures::{
        SinkExt, Stream,
        channel::mpsc::{self, Sender},
    },
    stream,
    widget::text,
    window::{self, Id},
};
use nvim_rs::{Handler, Neovim, compat::tokio::Compat, create};
use rmpv::Value;
use tracing::{error, trace, warn};
use tracing_subscriber::{Registry, fmt::Layer, layer::SubscriberExt};

use crate::window_manager::{AiEvent, AiWindow, WindowKind, WindowManager};

#[derive(Default)]
struct ShellApp {
    sender: Option<Sender<String>>,
    neovim: Option<Neovim<Compat<tokio::fs::File>>>,
    wm: WindowManager<Event>,
}

impl ShellApp {
    fn new() -> (Self, Task<Event>) {
        let (id, open) = window::open(window::Settings::default());
        (
            Self {
                sender: None,
                ..Default::default()
            },
            // Task::none(),
            open.map(move |id| Event::WindowOpened {
                id,
                kind: WindowKind::Ai,
            }),
        )
    }

    fn update(&mut self, message: Event) -> Task<Event> {
        match message {
            Event::Noop => Task::none(),
            Event::Todo => {
                trace!("message recieved");
                Task::none()
            }
            Event::NeovimHostReady(sender) => {
                trace!("nvim ready");
                self.sender = Some(sender.clone());
                // let handler = NvimHandler {};
                let handler = NvimHandler {
                    sender: Some(sender),
                };

                Task::perform(
                    async move {
                        trace!("creating parent");
                        match create::tokio::new_parent(handler).await {
                            Ok((nvim, io)) => {
                                trace!("spawnig io");
                                nvim
                            }
                            Err(e) => {
                                panic!("ee");
                            }
                        }
                    },
                    Event::NeovimClientReady,
                )
            }
            Event::OpenWindowRequested(kind) => {
                let (_id, open) = window::open(window::Settings::default());
                open.map(move |id| Event::WindowOpened { id, kind })
            }
            Event::WindowOpened { id, kind } => {
                let window = match kind {
                    WindowKind::Ai => AiWindow::default(),
                };
                self.wm.insert(id, window);
                Task::none()
            }
            Event::WindowClosed(id) => {
                self.wm.remove(&id);
                window::close(id)
            }
            Event::ChildEvent(id, child) => {
                self.wm.update(id, child);
                Task::none()
            }
            Event::NeovimClientReady(client) => {
                self.neovim = Some(client);
                Task::none()
            }
            Event::RequestSendNeovimCommand(s) => {
                if let Some(client) = self.neovim.clone() {
                    return Task::perform(async move { client.command(&s).await }, |_| Event::Noop);
                }
                Task::none()
            }
        }
    }

    fn view(&self, id: Id) -> Element<'_, Event> {
        // text("test").into()
        self.wm.view(id).into()
    }

    fn sub(&self) -> Subscription<Event> {
        Subscription::run(nvim_worker)
    }
}

#[derive(Default, Clone)]
enum Event {
    #[default]
    Noop,
    Todo,
    OpenWindowRequested(WindowKind),
    WindowOpened {
        id: Id,
        kind: WindowKind,
    },
    WindowClosed(Id),
    NeovimHostReady(Sender<String>),
    ChildEvent(Id, ChildEvent),
    NeovimClientReady(Neovim<Compat<tokio::fs::File>>),
    RequestSendNeovimCommand(String),
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Noop => write!(f, "Noop"),
            Self::Todo => write!(f, "Todo"),
            Self::OpenWindowRequested(arg0) => {
                f.debug_tuple("OpenWindowRequested").field(arg0).finish()
            }
            Self::WindowOpened { id, kind } => f
                .debug_struct("WindowOpened")
                .field("id", id)
                .field("kind", kind)
                .finish(),
            Self::WindowClosed(arg0) => f.debug_tuple("WindowClosed").field(arg0).finish(),
            Self::NeovimHostReady(arg0) => f.debug_tuple("NeovimHostReady").field(arg0).finish(),
            Self::ChildEvent(arg0, arg1) => {
                f.debug_tuple("ChildEvent").field(arg0).field(arg1).finish()
            }
            Self::NeovimClientReady(_arg0) => f.debug_tuple("NeovimClientReady").finish(),
            Self::RequestSendNeovimCommand(arg0) => f
                .debug_tuple("RequestSendNeovimCommand")
                .field(arg0)
                .finish(),
        }
    }
}

#[derive(Debug, Clone)]
enum ChildEvent {
    Ai(AiEvent),
}

fn main() -> iced::Result {
    std::panic::set_hook(Box::new(move |panic| {
        error!("----- Panic -----");
        error!("{}", panic);
    }));
    let logfile = File::create("log.txt").unwrap();
    // let writer = BufWriter::new(logfile);
    let file_layer = Layer::default()
        .with_writer(logfile)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_thread_names(true);

    let subscriber = Registry::default().with(file_layer);
    let _ = tracing::subscriber::set_global_default(subscriber);

    iced::daemon(ShellApp::new, ShellApp::update, ShellApp::view)
        .subscription(ShellApp::sub)
        .run();

    Ok(())
}

#[derive(Clone, Debug)]
pub struct NvimHandler {
    sender: Option<Sender<String>>,
}

#[async_trait]
impl Handler for NvimHandler {
    type Writer = Compat<tokio::fs::File>;

    #[tracing::instrument(skip(_neovim))]
    async fn handle_notify(
        &self,
        _name: String,
        _args: Vec<Value>,
        _neovim: Neovim<<Self as Handler>::Writer>,
    ) {
        trace!("handle notify");
        let mut s = self.sender.clone().expect("failed to clone sender");
        match _name.as_ref() {
            "test" => match _neovim.command(r#"!ls"#).await {
                Ok(_) => {
                    trace!("Send command ok");
                    s.send("test".to_string()).await;
                }
                Err(e) => error!("Send command error: {}", e),
            },
            "ai" => {
                s.send("ai".to_string()).await;
            }
            x => {
                warn!("unhandled value: {}", x);
                panic!("panic for now");
            }
        };
    }
}

#[tracing::instrument]
fn nvim_worker() -> impl Stream<Item = Event> {
    trace!("inside nvim_worker");

    stream::channel(100, async |mut output| {
        trace!("channel");
        let (sender, mut reciever) = mpsc::channel(100);

        let _ = output.send(Event::NeovimHostReady(sender)).await;

        loop {
            use iced_futures::futures::StreamExt;
            let input = reciever.select_next_some().await;

            #[allow(clippy::single_match)]
            let event = match input.as_str() {
                "test" => Event::Todo,
                "ai" => Event::OpenWindowRequested(WindowKind::Ai),
                _ => Event::Noop,
            };
            output.send(event).await;
        }
    })
}
