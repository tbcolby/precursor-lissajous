//! Pure Lissajous maths for the Precursor app: curve generation with the
//! shared Q16.16 trig, the preset table, and an incremental tracer. No Xous
//! dependencies; everything here runs under `cargo test` on the host.
//!
//! The curve is `x = sin(a*s + phase)`, `y = sin(b*s)`. Dividing `a` and `b`
//! by their gcd gives the same figure, which then closes after exactly one
//! turn of `s` (audit: the old code ran `lcm(a,b)` turns, retracing the
//! curve up to 81 times).

pub use precursor_appkit_core::fixed;
use precursor_appkit_core::fixed::{mul, sin, TURN};

/// Highest frequency offered in the menu and the ratio table.
pub const FREQ_MAX: i32 = 9;
/// Phase is edited in 16 steps per turn (22.5 degrees each).
pub const PHASE_STEPS: i32 = 16;
/// One phase step in [`TURN`] units.
pub const PHASE_UNIT: i32 = TURN / PHASE_STEPS;
/// Segments revealed per animation tick.
pub const SEGMENTS_PER_TICK: usize = 64;

/// Frequencies and phase of a figure. Phase is in [`TURN`] units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Params {
    pub a: i32,
    pub b: i32,
    pub phase: i32,
}

impl Params {
    /// Build with frequencies clamped to 1..=[`FREQ_MAX`] and phase wrapped.
    pub fn new(a: i32, b: i32, phase: i32) -> Self {
        Self { a: a.clamp(1, FREQ_MAX), b: b.clamp(1, FREQ_MAX), phase: phase.rem_euclid(TURN) }
    }

    pub fn step_a(&mut self, delta: i32) { self.a = (self.a + delta).clamp(1, FREQ_MAX); }

    pub fn step_b(&mut self, delta: i32) { self.b = (self.b + delta).clamp(1, FREQ_MAX); }

    /// Move the phase by whole [`PHASE_UNIT`] steps, wrapping.
    pub fn step_phase(&mut self, delta: i32) { self.phase = (self.phase + delta * PHASE_UNIT).rem_euclid(TURN); }

    /// Phase in whole degrees (0..360).
    pub fn phase_degrees(&self) -> i32 { (self.phase as i64 * 360 / TURN as i64) as i32 }

    /// Frequencies divided by their gcd. The figure is unchanged, but it now
    /// closes after exactly one turn.
    pub fn reduced(&self) -> (i32, i32) {
        let g = gcd(self.a, self.b);
        (self.a / g, self.b / g)
    }
}

/// Greatest common divisor (never 0 for positive input; `gcd(0,0)` is 1).
pub fn gcd(a: i32, b: i32) -> i32 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

/// A named starting point.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    pub name: &'static str,
    pub params: Params,
}

const fn preset(name: &'static str, a: i32, b: i32, quarter_turns: i32) -> Preset {
    Preset { name, params: Params { a, b, phase: quarter_turns * (TURN / 8) } }
}

/// Presets. Phase is given in eighth-turns (2 = 90 degrees).
pub const PRESETS: &[Preset] = &[
    preset("Circle", 1, 1, 2),
    preset("Figure-8", 1, 2, 0),
    preset("Bowtie", 2, 3, 0),
    preset("Trefoil", 3, 2, 2),
    preset("Clover", 3, 4, 0),
    preset("Star Knot", 5, 4, 2),
    preset("Diamond", 1, 2, 2),
    preset("Infinity", 2, 1, 2),
    preset("Ribbon", 3, 5, 1),
    preset("Atomic", 5, 6, 0),
];

/// A sampled Lissajous figure: `len()` segments between `len() + 1` points,
/// computed on demand (no point buffer, so memory is constant).
#[derive(Debug, Clone, Copy)]
pub struct Curve {
    a: i32,
    b: i32,
    phase: i32,
    center: (i32, i32),
    amp: (i32, i32),
    step: i32,
}

impl Curve {
    /// Figure with a step chosen so segments stay a few pixels long.
    pub fn new(p: Params, center: (i32, i32), amp: (i32, i32)) -> Self {
        let (a, b) = p.reduced();
        let want = (24 / a.max(b)).max(1);
        Self::with_step(p, center, amp, want)
    }

    /// Figure sampled every `step` turn units (rounded up to a power of two,
    /// at most [`TURN`], so the curve closes exactly).
    pub fn with_step(p: Params, center: (i32, i32), amp: (i32, i32), step: i32) -> Self {
        let (a, b) = p.reduced();
        let step = (step.clamp(1, TURN) as u32).next_power_of_two().min(TURN as u32) as i32;
        Self { a, b, phase: p.phase, center, amp, step }
    }

    /// Number of segments (one full period).
    pub fn len(&self) -> usize { (TURN / self.step) as usize }

    pub fn is_empty(&self) -> bool { false }

    /// Point `i`, for `i` in `0..=len()`; `point(len()) == point(0)`.
    pub fn point(&self, i: usize) -> (i32, i32) {
        let s = (i as i32) * self.step;
        (
            self.center.0 + mul(self.amp.0, sin(self.a * s + self.phase)),
            self.center.1 + mul(self.amp.1, sin(self.b * s)),
        )
    }

    /// Segments `range` (indices into `0..len()`), ready for `Screen::lines`.
    pub fn segments(&self, range: core::ops::Range<usize>) -> impl Iterator<Item = ((i32, i32), (i32, i32))> + '_ {
        let end = range.end.min(self.len());
        (range.start..end).map(move |i| (self.point(i), self.point(i + 1)))
    }
}

/// Reveals a curve a few segments at a time (the animated pen).
#[derive(Debug, Clone, Copy)]
pub struct Tracer {
    pub curve: Curve,
    drawn: usize,
}

impl Tracer {
    pub fn new(curve: Curve) -> Self { Self { curve, drawn: 0 } }

    /// Segments already revealed.
    pub fn drawn(&self) -> usize { self.drawn }

    pub fn done(&self) -> bool { self.drawn >= self.curve.len() }

    /// Reveal up to `n` more segments and return their index range. Each
    /// segment is returned exactly once, so incremental drawing never
    /// overdraws.
    pub fn advance(&mut self, n: usize) -> core::ops::Range<usize> {
        let start = self.drawn;
        self.drawn = (self.drawn + n).min(self.curve.len());
        start..self.drawn
    }
}

/// Segments for a thumbnail of `p` in a cell (used by the ratio table).
pub fn thumbnail(p: Params, center: (i32, i32), radius: i32) -> Curve {
    Curve::with_step(p, center, (radius, radius), TURN / 64)
}

#[cfg(test)]
mod tests;
