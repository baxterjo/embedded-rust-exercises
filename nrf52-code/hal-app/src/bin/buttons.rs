#![no_main]
#![no_std]

use core::time::Duration;

use cortex_m_rt::entry;
use dk::{Buttons, Leds, Timer};
// this imports `src/lib.rs`to retrieve our global logger + panicking-behavior
use hal_app as _;

const GAME_TICK: Duration = Duration::from_millis(20);
const TICK_MAX: Duration = Duration::from_millis(1000);
/// Must be twice game tick to prevent aliasing
const TICK_MIN: Duration = GAME_TICK
    .checked_mul(2)
    .expect("Min duration would overflow");
const TICK_STEP: Duration = Duration::from_millis(10);

#[entry]
fn main() -> ! {
    let board = dk::init().unwrap();
    let mut leds = board.leds;
    let mut buttons = board.buttons;

    let mut timer = board.timer;
    let rate = Duration::from_millis(500);
    let mut animation = Animation {
        state: AnimationState::default(),
        tick_rate: rate,
        soft_timer: SoftTimer::new(rate),
    };

    defmt::println!("Polling button every 20ms");
    loop {
        if buttons._1.is_pressed() {
            animation.state = AnimationState::Led1;
        }
        if buttons._2.is_pressed() {
            if animation.state != AnimationState::Paused {
                animation.state = AnimationState::Paused;
            } else {
                animation.state = AnimationState::Led1;
            }
        }
        if buttons._3.is_pressed() {
            animation.increase_speed(TICK_STEP);
        }
        if buttons._4.is_pressed() {
            animation.decrease_speed(TICK_STEP);
        }
        animation.tick();
        animation.set_leds(&mut leds);
        timer.wait(GAME_TICK);
    }
}

struct Animation {
    state: AnimationState,
    tick_rate: Duration,
    soft_timer: SoftTimer,
}

#[derive(Default, PartialEq)]
enum AnimationState {
    #[default]
    Off,
    Paused,
    Led1,
    Led2,
    Led3,
    Led4,
}

impl Animation {
    fn set_leds(&self, leds: &mut Leds) {
        match self.state {
            AnimationState::Off => {
                leds._1.off();
                leds._2.off();
                leds._3.off();
                leds._4.off();
            }
            AnimationState::Led1 => {
                leds._1.on();
                leds._2.off();
                leds._3.off();
                leds._4.off();
            }
            AnimationState::Led2 => {
                leds._1.off();
                leds._2.on();
                leds._3.off();
                leds._4.off();
            }
            AnimationState::Led3 => {
                leds._1.off();
                leds._2.off();
                leds._3.on();
                leds._4.off();
            }
            AnimationState::Led4 => {
                leds._1.off();
                leds._2.off();
                leds._3.off();
                leds._4.on();
            }
            AnimationState::Paused => {}
        }
    }

    fn increase_speed(&mut self, step: Duration) {
        self.tick_rate = self.tick_rate.saturating_add(step).min(TICK_MAX);
        self.soft_timer.change_deadline(self.tick_rate);
    }
    fn decrease_speed(&mut self, step: Duration) {
        self.tick_rate = self.tick_rate.saturating_sub(step).max(TICK_MIN);
        self.soft_timer.change_deadline(self.tick_rate);
    }
    fn tick(&mut self) {
        if self.soft_timer.elapsed() {
            match self.state {
                AnimationState::Led1 => self.state = AnimationState::Led2,
                AnimationState::Led2 => self.state = AnimationState::Led4,
                AnimationState::Led3 => self.state = AnimationState::Led1,
                AnimationState::Led4 => self.state = AnimationState::Led3,
                _ => {}
            }
            self.soft_timer.reset();
        }
    }
}

struct SoftTimer {
    start_time: Duration,
    duration: Duration,
}

impl SoftTimer {
    fn new(duration: Duration) -> Self {
        Self {
            start_time: Duration::from_micros(dk::uptime_us()),
            duration,
        }
    }

    fn elapsed(&self) -> bool {
        let now = Duration::from_micros(dk::uptime_us());
        now - self.start_time >= self.duration
    }

    fn reset(&mut self) {
        self.start_time = Duration::from_micros(dk::uptime_us());
    }
    /// Returns if the timer has elapsed with the new deadline.
    fn change_deadline(&mut self, new: Duration) -> bool {
        self.duration = new;
        self.elapsed()
    }
}
