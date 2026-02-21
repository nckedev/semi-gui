use std::time::Instant;

use iced::{
    Color, Point, Renderer, Size,
    border::Radius,
    widget::canvas::{self, Cache},
};

pub struct Spinner {
    bars: u32,
    start: Instant,
    now: Instant,
    cache: canvas::Cache,
    background: canvas::Cache,
    value: f32,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            bars: 50,
            start: Instant::now(),
            now: Instant::now(),
            cache: Cache::default(),
            background: Cache::default(),
            value: rand::random_range(5..30) as f32,
        }
    }

    pub fn update(&mut self, now: Instant) {
        self.now = now;
        self.value = rand::random_range(5..30) as f32;
        self.cache.clear();
    }
}

impl<M> canvas::Program<M> for Spinner {
    type State = u32;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let bg = self.background.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(Point::ORIGIN, frame.size(), Color::BLACK);
        });

        let spinner = self.cache.draw(renderer, bounds.size(), |frame| {
            let cy = frame.center().y;

            for x in 0..self.bars {
                let bar = canvas::Path::rounded_rectangle(
                    Point::new((x * 10) as f32, cy - (self.value / 2.)),
                    Size::new(5., self.value),
                    Radius::new(2.),
                );
                frame.fill(
                    &bar,
                    canvas::Fill {
                        style: canvas::Style::Solid(theme.palette().primary),
                        ..Default::default()
                    },
                );
            }
        });
        vec![bg, spinner]
    }
}
