use std::{
    fmt::Debug,
    ops::{Add, Mul, Sub},
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub struct Animated<T = f32>
where
    T: Add<T> + Sub<T> + Copy,
{
    status: AnimationStatus,
    current: T,
    start: T,
    target: T,
    og_start: T,
    og_target: T,
    start_time: Option<Instant>,
    duration: Duration,
    delta: T,
    delay: Duration,
}

// TODO: Sequence animations (keyframes?)

impl<T> Animated<T>
where
    T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Copy + Default + From<f32> + Debug,
{
    pub fn new(starting_value: f32) -> Self {
        Self {
            status: AnimationStatus::Idle,
            current: starting_value.into(),
            start: T::from(starting_value),
            target: T::default(),
            og_start: T::from(starting_value),
            og_target: T::default(),
            start_time: None,
            duration: Duration::from_millis(100),
            delta: T::default(),
            delay: Duration::from_millis(0),
        }
    }

    pub fn is_animating(&self) -> bool {
        matches!(self.status, AnimationStatus::Running(..))
    }

    pub fn update(&mut self, tick: Instant) {
        if self.status == AnimationStatus::Idle || self.status == AnimationStatus::Done {
            return;
        }
        let Some(start_time) = self.start_time else {
            self.status = AnimationStatus::Idle;
            return;
        };
        if tick - start_time >= self.duration + self.delay {
            // set to target to avoid rounding errors
            self.current = self.target;
            self.status = AnimationStatus::Done;
            return;
        }
        if tick - start_time < self.delay {
            return;
        }
        let elapsed = (tick - start_time) - self.delay;
        // if elapsed < self.delay {
        //     return;
        // }
        let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        self.status = AnimationStatus::Running(progress);
        assert!(progress <= 1.);
        self.current = self.start + (self.delta * T::from(progress));
    }

    // returns the progress of an animation [0,1]
    // if the animation has a delay that part is not included in the progress.
    // if an animations has an delay of 300 ms, progress sould return 0 for the first 300 ms.
    pub fn progress(&self) -> f32 {
        match self.status {
            AnimationStatus::Idle => 0.,
            AnimationStatus::Running(x) => x,
            AnimationStatus::Done => 1.,
        }
        // if self.status == AnimationStatus::Idle {
        //     return 1.;
        // }
        // if let Some(start) = self.start_time {
        //     let elapsed = (Instant::now() - start) - self.delay;
        //     let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        //     tracing::info!("progress: {}", progress);
        //     progress
        // } else {
        //     0.
        // }
    }

    pub fn animate_to(&mut self, target: T) -> &mut Self
    where
        T: Add<T> + Sub<T> + Copy + Default,
    {
        self.og_target = target;
        self.target = target;
        self
    }

    pub fn with_duration(&mut self, dur: u64) -> &mut Self {
        self.duration = Duration::from_millis(dur);
        self
    }

    pub fn with_delay(&mut self, delay: u64) -> &mut Self {
        self.delay = Duration::from_millis(delay);
        self
    }

    pub fn build(&mut self) -> Self {
        *self
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.target = self.og_target;
        self.delta = self.target - self.current;
        self.start = self.current;
        self.status = AnimationStatus::Running(0.);
        tracing::info!(
            "START    >> start: {:?}, delta: {:?}, target: {:?}",
            self.start,
            self.delta,
            self.target
        );
    }

    pub fn start_reverse(&mut self) {
        self.start_time = Some(Instant::now());
        self.target = self.og_start;
        self.delta = self.target - self.current;
        self.start = self.current;
        self.status = AnimationStatus::Running(0.);
        tracing::info!(
            "REVERSE  >> start: {:?}, delta: {:?}, target: {:?}",
            self.start,
            self.delta,
            self.target
        );
    }

    /// returns the current value
    pub fn value(&self) -> T {
        self.current
    }

    #[allow(dead_code)]
    pub fn value_ref(&self) -> &T {
        &self.current
    }
}

impl<T> From<Animated<T>> for f32
where
    T: Into<f32> + Copy + Mul + Add + Sub,
{
    fn from(value: Animated<T>) -> Self {
        value.current.into()
    }
}

// fn lerp<T>(a: T, b: T, t: T) -> T
// where
//     T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Copy + Default + From<f32> + Debug,
// {
//     assert!(t > T::from(0.) && t < T::from(1.));
//     a + (b - a) * t
// }
//
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AnimationStatus {
    Idle,
    Running(f32),
    Done,
}

enum AnimationProgress {
    Forward,
    Reverse,
}

pub enum AnimationRepeat {
    Once,
    Count(u32),
    Forever,
}

pub enum Easing {
    Linear,
    CubicIn,
    CubicOut,
}

pub struct AnimationCoordinator {
    values: Vec<(u32, Animated<f32>, bool)>,
    total_duration: u64,
}

pub enum AnimationKey {
    Percent(u32),
    Duration(u64),
}

impl AnimationCoordinator {
    pub fn new() -> Self {
        Self {
            values: vec![],
            total_duration: 0,
        }
    }

    pub fn add(mut self, key: u32, animation: Animated<f32>) -> Self {
        self.values.push((key, animation, false));
        self
    }

    pub fn start(&mut self) {
        for a in &mut self.values {
            if a.0 == 0 {
                a.1.start();
            }
        }
    }
}
