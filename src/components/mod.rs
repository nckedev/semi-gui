pub mod spinner;

use std::path::PathBuf;
use std::time::Duration;

use iced::widget::text::Fragment;
use iced::{
    Background, Border, Color, Element, Padding, Shadow, Theme, Vector,
    border::Radius,
    theme::palette,
    widget::{self, container, svg, tooltip},
};
use iced_ext::IcedExt;

type ColorFn = Option<Box<dyn Fn(&Theme) -> Color>>;

pub struct IconButton<'a, M> {
    path: PathBuf,
    size: u32,
    hover_color: ColorFn,
    color: ColorFn,
    tooltip_text: Option<Fragment<'a>>,
    tooltip_delay: u64,
    on_press: Option<M>,
    opacity: Option<f32>,
    text: Option<&'a str>,
}

#[allow(dead_code)]
impl<'a, M> IconButton<'a, M> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            size: 24,
            hover_color: None,
            tooltip_text: None,
            tooltip_delay: 300,
            on_press: None,
            color: None,
            opacity: None,
            text: None,
        }
    }

    pub fn size(mut self, size: u32) -> Self {
        self.size = size;
        self
    }

    pub fn hover_color(mut self, f: impl Fn(&Theme) -> Color + 'static) -> Self {
        self.hover_color = Some(Box::new(f));
        self
    }

    pub fn tooltip(mut self, text: impl Into<Fragment<'a>>, delay: u64) -> Self {
        self.tooltip_text = Some(text.into());
        self.tooltip_delay = delay;
        self
    }

    pub fn color(mut self, f: impl Fn(&Theme) -> Color + 'static) -> Self {
        self.color = Some(Box::new(f));
        self
    }

    pub fn on_press(mut self, event: M) -> Self {
        self.on_press = Some(event);
        self
    }

    pub fn on_press_maybe(mut self, event: Option<M>) -> Self {
        self.on_press = event;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    pub fn with_text_maybe(mut self, text: Option<&'a str>) -> Self {
        self.text = text;
        self
    }

    pub fn view(self) -> Element<'a, M>
    where
        M: Clone + 'a,
    {
        let icon = widget::mouse_area(
            widget::Svg::from_path(&self.path)
                .height(self.size)
                .width(self.size)
                .style(move |t: &Theme, s| {
                    let color = match s {
                        svg::Status::Idle => {
                            if let Some(color) = &self.color {
                                (color)(t)
                            } else {
                                let mut c = palette::lighten(t.palette().text, 0.35);
                                if let Some(op) = self.opacity {
                                    // BUG: this is a bug? alpha not applying to svgs
                                    c = Color::from_rgba(c.r, c.g, c.b, op);
                                }
                                c
                            }
                        }
                        svg::Status::Hovered => {
                            if let Some(hc) = &self.hover_color {
                                (hc)(t)
                            } else {
                                t.palette().text
                            }
                        }
                    };
                    svg::Style { color: Some(color) }
                }),
        );

        let icon = if let Some(press) = self.on_press {
            icon.on_press(press)
        } else {
            icon
        };

        let icon = if let Some(text) = self.text {
            let text_style = |t: &Theme| widget::text::Style {
                color: Some(t.palette().text),
            };
            widget::row![
                icon,
                widget::space().width(5),
                widget::text(text).style(text_style)
            ]
        } else {
            widget::row![icon]
        };

        let icon: Element<'_, M> = if let Some(tooltip_text) = self.tooltip_text {
            tooltip(icon, tp(tooltip_text), tooltip::Position::Top)
                .delay(Duration::from_millis(self.tooltip_delay))
                .into()
        } else {
            icon.into()
        };

        icon
    }
}

fn tp<'a, M>(msg: Fragment<'a>) -> widget::Container<'a, M> {
    widget::text(msg)
        .container()
        .padding(Padding::new(0.).horizontal(7).vertical(4))
        .style(|t: &Theme| container::Style {
            background: Some(Background::Color(t.palette().background)),
            border: Border {
                color: t.palette().background,
                width: 1.,
                radius: Radius::new(5),
            },
            shadow: Shadow {
                color: Color::from_rgba(0., 0., 0., 0.5),
                offset: Vector::new(1., 1.),
                blur_radius: 5.,
            },
            ..Default::default()
        })
}

impl<'a, M: Clone + 'a> From<IconButton<'a, M>> for Element<'a, M> {
    fn from(val: IconButton<'a, M>) -> Self {
        val.view()
    }
}
