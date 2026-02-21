mod agent;
mod ai;
mod animated;
mod commands;
mod components;
mod nvim_types;
mod prompt;
mod settings;
mod strings;
mod style;
mod window_manager;

use std::{
    fmt::Debug,
    fs::File,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use dotenv::dotenv;
use iced::{
    Element, Subscription, Task, Theme, Window, color,
    futures::{
        SinkExt, Stream,
        channel::mpsc::{self, Sender},
    },
    keyboard::{self, Key, Modifiers, key::Named},
    stream,
    theme::{Custom, Palette},
    window::{self, Id, settings::PlatformSpecific},
};
use iced_futures::backend::default::time;
use nvim_rs::{Handler, Neovim, compat::tokio::Compat, create};
use rmpv::Value;
use tracing::{error, trace, warn};
use tracing_subscriber::{Registry, filter, fmt::Layer, layer::SubscriberExt};

use crate::{
    agent::agent::AgentEvent,
    ai::ai::{AiClient, ClientBuilder, Request, Response},
    nvim_types::NvimResponse,
    prompt::Prompt,
    window_manager::{SgWindow, WindowKind, WindowManager},
};

struct ShellApp {
    sender: Option<Sender<String>>,
    neovim: Option<Neovim<Compat<tokio::fs::File>>>,
    aiclient: Box<dyn AiClient>,
    wm: WindowManager,
    is_animating: bool,
    focused_window: Option<Id>,
}

impl ShellApp {
    fn new() -> (Self, Task<Event>) {
        let (_id, open) = window::open(window::Settings {
            platform_specific: PlatformSpecific {
                fullsize_content_view: true,
                title_hidden: false,
                titlebar_transparent: true,
            },
            ..Default::default()
        });
        (
            Self {
                sender: None,
                aiclient: ClientBuilder::openai().expect("failed to create open ai client "),
                neovim: None,
                wm: WindowManager::default(),
                is_animating: false,
                focused_window: None,
            },
            // Task::none(),
            open.map(move |id| Event::WindowOpened {
                id,
                kind: WindowKind::Agent,
            }),
        )
    }

    fn theme(&self, _window: Id) -> Option<Theme> {
        Some(Theme::Custom(Arc::new(Custom::new(
            "test".to_string(),
            Palette {
                background: color!(0xfffdf9),
                text: color!(0x181818),
                primary: color!(0xeba431),
                success: color!(0x00ff00),
                warning: color!(0xf00000),
                danger: color!(0xff0000),
            },
        ))))
    }

    /// the main update loop
    fn update(&mut self, message: Event) -> Task<Event> {
        match message {
            Event::Animate => {
                self.is_animating = true;
                Task::none()
            }
            Event::Noop => Task::none(),
            Event::Error(e) => {
                error!(e);
                Task::none()
            }
            Event::Todo => {
                trace!("message recieved");
                Task::none()
            }
            Event::Tick(instant) => {
                // TODO: this needs to be better
                self.wm.animate(instant);
                Task::none()
            }
            Event::NeovimWorkerReady(sender) => {
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
                            Ok((nvim, _io)) => {
                                trace!("spawnig io");
                                nvim
                            }
                            Err(e) => {
                                tracing::error!("error in parent: {}", e);
                                panic!("ee");
                            }
                        }
                    },
                    Event::NeovimClientReady,
                )
            }
            Event::OpenWindowRequested(kind) => {
                match kind {
                    // settings can only have once instance
                    WindowKind::Settings => {
                        if let Some(id) = self.wm.find_first(kind) {
                            return window::gain_focus(id);
                        }
                    }
                    _ => {}
                }
                let (_id, open) = window::open(window::Settings::default());
                open.map(move |id| Event::WindowOpened { id, kind })
            }
            Event::WindowOpened { id, kind } => {
                // let window = match kind {
                //     WindowKind::Agent => SgWindow::sg_window_from_kind(id, kind),
                // };
                let window = SgWindow::sg_window_from_kind(id, kind);
                self.wm.insert(id, window);
                Task::none()
            }
            Event::WindowClosed(id) => window::close(id),
            Event::ChildEvent(id, child) => self.wm.update(id, child),
            Event::NeovimClientReady(client) => {
                self.neovim = Some(client);
                Task::none()
            }
            Event::RequestSendNeovimCommand {
                command,
                arg,
                id,
                callback,
            } => {
                if let Some(client) = self.neovim.clone() {
                    return Task::perform(
                        async move {
                            if let Some(arg) = arg {
                                match client.command_output(&format!("{} {}", command, arg)).await {
                                    Ok(s) => callback(s, id),
                                    Err(e) => {
                                        tracing::error!("{}", e);
                                        // TODO: These errors need to be dispatched to the right
                                        // WindowKind and id somehow, for correct reporting
                                        AgentEvent::ErrorHappend(
                                            "Error with the neovim connection".to_string(),
                                        )
                                        .into_event(id)
                                    }
                                }
                            } else {
                                match client.command(&command).await {
                                    Ok(_) => callback("tset".to_string(), id),
                                    Err(e) => Event::Error(format!(
                                        "Error when sending command to neovim: {}",
                                        e
                                    )),
                                }
                            }
                        },
                        |event| event,
                    );
                }
                Task::none()
            }
            Event::AskClanker(id, kind, prev_id, prompt, callback) => {
                tracing::trace!("asking the clanker");
                let ai_client = self.aiclient.clone_box();
                let neovim_client = self.neovim.clone();
                // TODO: do the prompt transformation here, include files, lsp queries etc...
                let task1 = Task::perform(
                    async move {
                        // build the prompt
                        let mut buf = String::default();
                        for segment in prompt.segments {
                            match segment.kind {
                                prompt::SegmentKind::Text => {
                                    buf.push_str(segment.expand(&prompt.raw))
                                }
                                prompt::SegmentKind::File => buf.push_str("file"),
                                prompt::SegmentKind::Lsp => buf.push_str("lsp"),
                                prompt::SegmentKind::Selected => {
                                    if let Some(ref client) = neovim_client {
                                        match client.command_output("SgGet selected").await {
                                            Ok(r) => {
                                                let response: NvimResponse =
                                                    match serde_json::from_str(&r) {
                                                        Ok(r) => r,
                                                        Err(e) => {
                                                            tracing::error!("{}", e);
                                                            return kind.create_user_error(
                                                                "Something went wrong with deserialization".to_string(),
                                                                id,
                                                            );
                                                        }
                                                    };
                                                tracing::info!("response from neovim : {}", &r);
                                                tracing::info!("json: {:#?}", response);

                                                buf = response.content_as_md();
                                            }
                                            Err(e) => {
                                                // neovim call err
                                                tracing::error!("Error from neovim : {}", &e);
                                                return kind.create_user_error(
                                                    format!("neovim error: {}", e),
                                                    id,
                                                );
                                            }
                                        }
                                    }
                                }
                            };
                        }
                        let mut request = Request::new(buf);
                        request.prev_id = prev_id;
                        tracing::info!("request: {:?}", &request);
                        match ai_client.send(request).await {
                            Ok(r) => callback(r, id),
                            Err(e) => {
                                tracing::error!("{:?}", e);
                                panic!("send request fail")
                            }
                        }
                    },
                    //event is the callback
                    move |event| event,
                );

                Task::batch([task1])
            }
            Event::WindowEvent(id, event) => {
                // tracing::info!("{:?}", event);
                match event {
                    window::Event::Closed => {
                        self.wm.remove(&id);
                    }
                    window::Event::RedrawRequested(_instant) => {}
                    window::Event::CloseRequested => {}
                    window::Event::Focused => self.focused_window = Some(id),
                    window::Event::Unfocused => self.focused_window = None,
                    _ => {}
                }
                Task::none()
            }
            Event::KbdEvent(event) => {
                let Some(id) = self.focused_window else {
                    return Task::none();
                };
                if self.wm.is_agent(id) {
                    match event {
                        keyboard::Event::KeyPressed {
                            key,
                            modifiers: Modifiers::LOGO,
                            ..
                        } => {
                            if key == Key::Named(Named::Enter) {
                                tracing::info!("mac enter pressed");
                                return self
                                    .wm
                                    .update(id, ChildEvent::Agent(AgentEvent::PressedSend));
                            }
                            if key == Key::Character("p".into()) {
                                tracing::info!("p pressed");
                            }
                        }
                        keyboard::Event::KeyReleased { .. } => {}
                        keyboard::Event::ModifiersChanged(_modifiers) => {}
                        _ => {}
                    }
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
        let kbd_event = keyboard::listen().map(Event::KbdEvent);
        let window_event = window::events().map(|(id, event)| Event::WindowEvent(id, event));
        let nvim = Subscription::run(nvim_worker);
        if self.wm.is_animating() {
            let ticker = time::every(Duration::from_millis(16)).map(Event::Tick);
            return Subscription::batch([nvim, ticker, window_event, kbd_event]);
        }
        Subscription::batch([nvim, window_event, kbd_event])
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Default, Clone)]
enum Event {
    #[default]
    Noop,
    Error(String),
    Todo,
    Tick(Instant),
    OpenWindowRequested(WindowKind),
    WindowOpened {
        id: Id,
        kind: WindowKind,
    },
    WindowClosed(Id),
    NeovimWorkerReady(Sender<String>),
    ChildEvent(Id, ChildEvent),
    NeovimClientReady(Neovim<Compat<tokio::fs::File>>),
    RequestSendNeovimCommand {
        command: String,
        arg: Option<String>,
        id: Id,
        callback: fn(String, Id) -> Event,
    },
    AskClanker(
        Id,
        WindowKind,
        Option<String>,
        Prompt,
        fn(Response, Id) -> Event,
    ),
    Animate,
    WindowEvent(Id, iced::window::Event),
    KbdEvent(iced::keyboard::Event),
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Noop => write!(f, "Noop"),
            Event::Error(e) => f.debug_tuple("Error").field(e).finish(),
            Self::Tick(instant) => f.debug_tuple("Tick").field(instant).finish(),
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
            Self::NeovimWorkerReady(arg0) => f.debug_tuple("NeovimHostReady").field(arg0).finish(),
            Self::ChildEvent(arg0, arg1) => {
                f.debug_tuple("ChildEvent").field(arg0).field(arg1).finish()
            }
            Self::NeovimClientReady(_arg0) => f.debug_tuple("NeovimClientReady").finish(),
            Self::RequestSendNeovimCommand { .. } => {
                f.debug_tuple("RequestSendNeovimCommand").finish()
            }
            Self::AskClanker(_, _, _, _, _) => write!(f, "clanker"),
            _ => write!(f, "debug not implemented"),
        }
    }
}

#[derive(Debug, Clone)]
enum ChildEvent {
    Agent(AgentEvent),
    Chat,
}

fn main() -> iced::Result {
    dotenv().ok();
    std::panic::set_hook(Box::new(move |panic| {
        error!("----- Panic -----");
        error!("{}", panic);
    }));

    let logfile = File::create("log.txt").unwrap();

    let filter = filter::Targets::new()
        .with_target("winit", tracing::Level::INFO)
        .with_default(tracing::Level::DEBUG);

    let file_layer = Layer::default()
        .with_writer(logfile)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_thread_names(true);

    let subscriber = Registry::default().with(file_layer).with(filter);
    let _ = tracing::subscriber::set_global_default(subscriber);

    iced::daemon(ShellApp::new, ShellApp::update, ShellApp::view)
        .theme(ShellApp::theme)
        .subscription(ShellApp::sub)
        .run()
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
        name: String,
        args: Vec<Value>,
        _neovim: Neovim<<Self as Handler>::Writer>,
    ) {
        trace!("handle notify");
        let mut sender = self.sender.clone().expect("failed to clone sender");
        let value = match name.as_ref() {
            "test" => "test".to_string(),
            "ai" => "ai".to_string(),
            "init" => "init".to_string(),
            x => {
                warn!("unhandled value: {}", x);
                x.to_string()
            }
        };

        if let Err(e) = sender.send(value).await {
            error!("Send error: {} ", e)
        }
    }
}

#[tracing::instrument]
fn nvim_worker() -> impl Stream<Item = Event> {
    trace!("inside nvim_worker");

    stream::channel(100, async |mut output| {
        trace!("channel");
        let (sender, mut reciever) = mpsc::channel(100);

        let _ = output.send(Event::NeovimWorkerReady(sender)).await;

        loop {
            use iced_futures::futures::StreamExt;
            let input = reciever.select_next_some().await;

            #[allow(clippy::single_match)]
            let event = match input.as_str() {
                "test" => Event::Todo,
                "ai" => Event::OpenWindowRequested(WindowKind::Agent),
                _ => Event::Noop,
            };
            output.send(event).await;
        }
    })
}
