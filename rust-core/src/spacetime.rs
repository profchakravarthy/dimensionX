//! Universal Space-Time Clock, Slow-Motion & Hyper-Warp Time Dilation (`REQ-CORE-002`).
//!
//! Maintains Universal Proper Time `T` (in seconds) with a base rate of
//! `1.0 simulation second per 1.0 real second` (`time_scale = 1.0`) and fixed simulation
//! quantum `fixed_delta_time` (default `1/60 s`).
//!
//! Supports:
//! - Pausing (`time_scale = 0.0`), single-stepping (`step_single_tick()`),
//! - **Slow-motion sub-1x proper time** (`0.05x, 0.1x, 0.25x, 0.5x`),
//! - **Hyper-warp >1x proper time** (`2x, 5x, 10x, 20x, 50x, 100x, 250x, 500x, 1000x`),
//! - Dedicated `-` (`slow_down_time()`) and `+` (`speed_up_time()`) step controls,
//! - Fractional sub-tick `interpolation_alpha` for ultra-smooth slow-motion rendering.
//!
//! 100% pure Rust standard library (`std`), MIT License.

/// Canonical ordered time-scale ladder spanning paused (`0.0`), slow-motion (`0.05..0.5`),
/// real-time (`1.0`), and hyper-warp (`2.0..1000.0`).
pub const TIME_SCALE_LADDER: &[f64] = &[
    0.0, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 250.0, 500.0, 1000.0,
];

/// Telemetry returned by `step_single_tick` and `advance_real_time`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepResult {
    pub ticks_executed: usize,
    pub delta_time: f64,
    pub simulated_time_advanced: f64,
    pub universal_time: f64,
    pub tick_count: u64,
    pub interpolation_alpha: f64,
}

/// Deterministic space-time accumulator and proper-time dilation controller (`REQ-CORE-002`).
#[derive(Clone, Debug)]
pub struct SpaceTimeClock {
    pub universal_time: f64,
    pub tick_count: u64,
    pub fixed_delta_time: f64,
    pub time_scale: f64,
    pub previous_time_scale: f64,
    pub max_sub_steps_per_frame: usize,
    pub paused: bool,
    pub accumulator: f64,
}

impl SpaceTimeClock {
    /// Creates a new `SpaceTimeClock`.
    pub fn new(fixed_delta_time: f64, time_scale: f64, max_sub_steps_per_frame: usize) -> Self {
        let dt = if fixed_delta_time > 0.0 && fixed_delta_time.is_finite() {
            fixed_delta_time
        } else {
            1.0 / 60.0
        };
        let scale = if time_scale >= 0.0 && time_scale.is_finite() {
            time_scale
        } else {
            1.0
        };
        let prev = if scale > 0.0 { scale } else { 1.0 };

        Self {
            universal_time: 0.0,
            tick_count: 0,
            fixed_delta_time: dt,
            time_scale: scale,
            previous_time_scale: prev,
            max_sub_steps_per_frame: max_sub_steps_per_frame.max(1),
            paused: scale == 0.0,
            accumulator: 0.0,
        }
    }

    /// Sets the simulation time multiplier (`0.0` = paused, `< 1.0` = slow-motion, `1.0` = 1s/s, `> 1.0` = hyper-warp).
    pub fn set_time_scale(&mut self, scale: f64) {
        let clamped = if scale.is_finite() && scale >= 0.0 {
            scale
        } else {
            0.0
        };
        if clamped > 0.0 {
            self.previous_time_scale = clamped;
        }
        self.time_scale = clamped;
        self.paused = clamped == 0.0;
    }

    /// **`-` button logic:** Steps down to the next lower value in `TIME_SCALE_LADDER`
    /// (allowing smooth slow-down below `1.0x` into `0.5x -> 0.25x -> 0.1x -> 0.05x -> 0.0x`).
    pub fn slow_down_time(&mut self) -> f64 {
        let current = if self.paused { 0.0 } else { self.time_scale };
        let mut next_scale = 0.0;
        for &val in TIME_SCALE_LADDER.iter().rev() {
            if val < current - 1e-6 {
                next_scale = val;
                break;
            }
        }
        self.set_time_scale(next_scale);
        self.time_scale
    }

    /// **`+` button logic:** Steps up to the next higher value in `TIME_SCALE_LADDER`
    /// (`0.0x -> 0.05x -> 0.1x -> 0.25x -> 0.5x -> 1.0x -> 2.0x -> 5.0x -> ... -> 1000.0x`).
    pub fn speed_up_time(&mut self) -> f64 {
        let current = if self.paused { 0.0 } else { self.time_scale };
        let mut next_scale = *TIME_SCALE_LADDER.last().unwrap_or(&1000.0);
        for &val in TIME_SCALE_LADDER {
            if val > current + 1e-6 {
                next_scale = val;
                break;
            }
        }
        self.set_time_scale(next_scale);
        self.time_scale
    }

    /// Pauses simulation time progression while remembering the last active non-zero `time_scale`.
    pub fn pause(&mut self) {
        if self.time_scale > 0.0 {
            self.previous_time_scale = self.time_scale;
        }
        self.time_scale = 0.0;
        self.paused = true;
    }

    /// Resumes simulation time progression at the previously active non-zero `time_scale`.
    pub fn resume(&mut self) {
        let target = if self.previous_time_scale > 0.0 {
            self.previous_time_scale
        } else {
            1.0
        };
        self.set_time_scale(target);
    }

    /// Advances Universal Proper Time by exactly one fixed simulation quantum (`fixed_delta_time`), even when paused.
    pub fn step_single_tick<F: FnMut(f64, f64, u64)>(&mut self, mut on_step: F) -> StepResult {
        self.tick_count += 1;
        self.universal_time = (self.tick_count as f64) * self.fixed_delta_time;
        on_step(self.fixed_delta_time, self.universal_time, self.tick_count);
        let interpolation_alpha = (self.accumulator / self.fixed_delta_time).clamp(0.0, 1.0);
        StepResult {
            ticks_executed: 1,
            delta_time: self.fixed_delta_time,
            simulated_time_advanced: self.fixed_delta_time,
            universal_time: self.universal_time,
            tick_count: self.tick_count,
            interpolation_alpha,
        }
    }

    /// Advances simulation time deterministically based on elapsed real-world wall-clock seconds.
    /// Computes fractional `interpolation_alpha` in `[0.0, 1.0]` for smooth sub-tick rendering in slow-motion `< 1x`.
    pub fn advance_real_time<F: FnMut(f64, f64, u64)>(
        &mut self,
        real_delta_seconds: f64,
        mut on_step: F,
    ) -> StepResult {
        if !real_delta_seconds.is_finite()
            || real_delta_seconds <= 0.0
            || self.paused
            || self.time_scale <= 0.0
        {
            let interpolation_alpha = (self.accumulator / self.fixed_delta_time).clamp(0.0, 1.0);
            return StepResult {
                ticks_executed: 0,
                delta_time: self.fixed_delta_time,
                simulated_time_advanced: 0.0,
                universal_time: self.universal_time,
                tick_count: self.tick_count,
                interpolation_alpha,
            };
        }

        self.accumulator += real_delta_seconds * self.time_scale;
        let raw_ticks = ((self.accumulator + 1e-11) / self.fixed_delta_time).floor() as usize;
        let ticks_to_run = raw_ticks.min(self.max_sub_steps_per_frame);

        if ticks_to_run == 0 {
            let interpolation_alpha = (self.accumulator / self.fixed_delta_time).clamp(0.0, 1.0);
            return StepResult {
                ticks_executed: 0,
                delta_time: self.fixed_delta_time,
                simulated_time_advanced: 0.0,
                universal_time: self.universal_time,
                tick_count: self.tick_count,
                interpolation_alpha,
            };
        }

        self.accumulator =
            (self.accumulator - (ticks_to_run as f64) * self.fixed_delta_time).max(0.0);

        for _ in 0..ticks_to_run {
            self.tick_count += 1;
            self.universal_time = (self.tick_count as f64) * self.fixed_delta_time;
            on_step(self.fixed_delta_time, self.universal_time, self.tick_count);
        }

        let interpolation_alpha = (self.accumulator / self.fixed_delta_time).clamp(0.0, 1.0);
        StepResult {
            ticks_executed: ticks_to_run,
            delta_time: self.fixed_delta_time,
            simulated_time_advanced: (ticks_to_run as f64) * self.fixed_delta_time,
            universal_time: self.universal_time,
            tick_count: self.tick_count,
            interpolation_alpha,
        }
    }

    /// Resets Universal Proper Time, tick counter, and accumulator back to initial state.
    pub fn reset(&mut self) {
        self.universal_time = 0.0;
        self.tick_count = 0;
        self.accumulator = 0.0;
        self.paused = self.time_scale == 0.0;
    }
}
