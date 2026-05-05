//! # Trading Signals
//!
//! Layer above [`crate::indicators`] that turns raw indicator values into
//! discrete trading events ([`SignalEvent`]). Implementations are streaming
//! (`fn next(value) -> Option<SignalEvent>`) so they compose naturally with
//! the indicator API.
//!
//! ## Built-in signals
//!
//! - [`CrossUp`] / [`CrossDown`]: the classic two-series crossover
//!   (e.g. fast MA crossing the slow MA).
//! - [`ThresholdAbove`] / [`ThresholdBelow`]: the value crosses a fixed
//!   level (e.g. RSI breaching 70/30).
//! - [`Breakout`]: the value moves outside a rolling-window high/low
//!   (driven by [`crate::indicators::volatility::DonchianChannels`] or any
//!   custom upper/lower).
//!
//! Combinators ([`SignalExt::and`], [`SignalExt::or`], [`SignalExt::not`])
//! let users compose signals without writing custom structs.
//!
//! ## Example
//!
//! ```
//! use rsta::indicators::Indicator;
//! use rsta::indicators::trend::SimpleMovingAverage;
//! use rsta::signals::{CrossUp, Signal, SignalEvent};
//!
//! let mut fast = SimpleMovingAverage::new(3).unwrap();
//! let mut slow = SimpleMovingAverage::new(5).unwrap();
//! let mut cross = CrossUp::new();
//!
//! let prices = [10.0, 9.0, 8.0, 7.0, 8.0, 9.0, 11.0, 13.0, 15.0];
//! for &p in &prices {
//!     let f = fast.next(p).unwrap();
//!     let s = slow.next(p).unwrap();
//!     if let (Some(f), Some(s)) = (f, s) {
//!         if let Some(SignalEvent::Long) = cross.next((f, s)) {
//!             println!("fast crossed above slow at {}", p);
//!         }
//!     }
//! }
//! ```

/// A discrete trading event emitted by a [`Signal`].
///
/// `Long` and `Short` indicate an entry direction. `Exit` flags an explicit
/// reason to flatten an open position. `Hold` means "no signal this bar"
/// — most signals return `None` instead, but combinators emit `Hold` when
/// they need to distinguish "evaluated, no event" from "not enough data
/// yet".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalEvent {
    /// Open or maintain a long position.
    Long,
    /// Open or maintain a short position.
    Short,
    /// Exit any open position.
    Exit,
    /// Explicit "no actionable signal".
    Hold,
}

/// Streaming signal contract.
///
/// Implementations consume one input per bar and return an optional
/// [`SignalEvent`]. Returning `None` means "not enough state to decide
/// yet" (typically during warmup); returning `Some(SignalEvent::Hold)`
/// means "evaluated but nothing to do".
pub trait Signal {
    /// Type fed into the signal each bar (e.g. `(f64, f64)` for a
    /// two-series crossover, `f64` for a single threshold check).
    type Input;

    /// Process the next bar and return the resulting event (if any).
    fn next(&mut self, value: Self::Input) -> Option<SignalEvent>;

    /// Reset the internal state.
    fn reset(&mut self);
}

/// Extension methods for composing signals.
pub trait SignalExt: Signal + Sized {
    /// Logical AND: emit `Long`/`Short` only if **both** legs agree on
    /// the same direction this bar. Other combinations emit `Hold`.
    fn and<O>(self, other: O) -> AndSignal<Self, O>
    where
        O: Signal<Input = Self::Input>,
    {
        AndSignal { a: self, b: other }
    }

    /// Logical OR: emit the first non-`None`, non-`Hold` event from
    /// either leg. If both fire and they disagree, prefer the left leg.
    fn or<O>(self, other: O) -> OrSignal<Self, O>
    where
        O: Signal<Input = Self::Input>,
    {
        OrSignal { a: self, b: other }
    }

    /// Logical NOT: flip `Long` ↔ `Short`, leave `Exit` / `Hold` /
    /// `None` unchanged. Useful for inverting a signal cheaply.
    fn not(self) -> NotSignal<Self> {
        NotSignal { inner: self }
    }
}

impl<S: Signal + Sized> SignalExt for S {}

// ---------------------------------------------------------------------------
// Crossovers
// ---------------------------------------------------------------------------

/// Emit [`SignalEvent::Long`] the bar after `a` crosses **above** `b`.
///
/// Inputs are `(a, b)` tuples — typically two indicator outputs. The signal
/// only fires on the strict transition: equal values do not trigger.
#[derive(Debug, Default)]
pub struct CrossUp {
    prev: Option<(f64, f64)>,
}

impl CrossUp {
    /// Create a new CrossUp detector.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Signal for CrossUp {
    type Input = (f64, f64);
    fn next(&mut self, (a, b): (f64, f64)) -> Option<SignalEvent> {
        let event = match self.prev {
            Some((pa, pb)) if pa <= pb && a > b => Some(SignalEvent::Long),
            Some(_) => Some(SignalEvent::Hold),
            None => None,
        };
        self.prev = Some((a, b));
        event
    }
    fn reset(&mut self) {
        self.prev = None;
    }
}

/// Emit [`SignalEvent::Short`] the bar after `a` crosses **below** `b`.
#[derive(Debug, Default)]
pub struct CrossDown {
    prev: Option<(f64, f64)>,
}

impl CrossDown {
    /// Create a new CrossDown detector.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Signal for CrossDown {
    type Input = (f64, f64);
    fn next(&mut self, (a, b): (f64, f64)) -> Option<SignalEvent> {
        let event = match self.prev {
            Some((pa, pb)) if pa >= pb && a < b => Some(SignalEvent::Short),
            Some(_) => Some(SignalEvent::Hold),
            None => None,
        };
        self.prev = Some((a, b));
        event
    }
    fn reset(&mut self) {
        self.prev = None;
    }
}

// ---------------------------------------------------------------------------
// Thresholds
// ---------------------------------------------------------------------------

/// Emit [`SignalEvent::Long`] the bar after the input crosses **above** the
/// configured `level`. Useful for things like "RSI breaks 30 from below".
#[derive(Debug)]
pub struct ThresholdAbove {
    level: f64,
    prev: Option<f64>,
}

impl ThresholdAbove {
    /// Create a threshold detector that triggers on upward crossings of `level`.
    pub fn new(level: f64) -> Self {
        Self { level, prev: None }
    }
}

impl Signal for ThresholdAbove {
    type Input = f64;
    fn next(&mut self, value: f64) -> Option<SignalEvent> {
        let event = match self.prev {
            Some(prev) if prev <= self.level && value > self.level => Some(SignalEvent::Long),
            Some(_) => Some(SignalEvent::Hold),
            None => None,
        };
        self.prev = Some(value);
        event
    }
    fn reset(&mut self) {
        self.prev = None;
    }
}

/// Emit [`SignalEvent::Short`] the bar after the input crosses **below** the
/// configured `level`. Mirror of [`ThresholdAbove`].
#[derive(Debug)]
pub struct ThresholdBelow {
    level: f64,
    prev: Option<f64>,
}

impl ThresholdBelow {
    /// Create a threshold detector that triggers on downward crossings of `level`.
    pub fn new(level: f64) -> Self {
        Self { level, prev: None }
    }
}

impl Signal for ThresholdBelow {
    type Input = f64;
    fn next(&mut self, value: f64) -> Option<SignalEvent> {
        let event = match self.prev {
            Some(prev) if prev >= self.level && value < self.level => Some(SignalEvent::Short),
            Some(_) => Some(SignalEvent::Hold),
            None => None,
        };
        self.prev = Some(value);
        event
    }
    fn reset(&mut self) {
        self.prev = None;
    }
}

// ---------------------------------------------------------------------------
// Breakout (rolling channel)
// ---------------------------------------------------------------------------

/// Emit a breakout when the input rises above an upper level (long) or falls
/// below a lower level (short). Inputs are `(value, upper, lower)` tuples —
/// caller is responsible for feeding the channel from
/// [`crate::indicators::volatility::DonchianChannels`] or similar.
#[derive(Debug, Default)]
pub struct Breakout {
    inside: Option<bool>,
}

impl Breakout {
    /// Create a new breakout detector.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Signal for Breakout {
    type Input = (f64, f64, f64); // (value, upper, lower)
    fn next(&mut self, (value, upper, lower): (f64, f64, f64)) -> Option<SignalEvent> {
        let was_inside = self.inside;
        let now_inside = value <= upper && value >= lower;
        self.inside = Some(now_inside);
        match was_inside {
            None => None, // first bar: warmup
            Some(true) if value > upper => Some(SignalEvent::Long),
            Some(true) if value < lower => Some(SignalEvent::Short),
            _ => Some(SignalEvent::Hold),
        }
    }
    fn reset(&mut self) {
        self.inside = None;
    }
}

// ---------------------------------------------------------------------------
// Combinators
// ---------------------------------------------------------------------------

/// Logical AND of two signals — see [`SignalExt::and`].
#[derive(Debug)]
pub struct AndSignal<A, B> {
    a: A,
    b: B,
}

impl<A, B> Signal for AndSignal<A, B>
where
    A: Signal,
    B: Signal<Input = A::Input>,
    A::Input: Clone,
{
    type Input = A::Input;
    fn next(&mut self, value: Self::Input) -> Option<SignalEvent> {
        match (self.a.next(value.clone()), self.b.next(value)) {
            (Some(SignalEvent::Long), Some(SignalEvent::Long)) => Some(SignalEvent::Long),
            (Some(SignalEvent::Short), Some(SignalEvent::Short)) => Some(SignalEvent::Short),
            (Some(SignalEvent::Exit), _) | (_, Some(SignalEvent::Exit)) => Some(SignalEvent::Exit),
            (Some(_), Some(_)) => Some(SignalEvent::Hold),
            // If either leg is still warming up, abstain.
            _ => None,
        }
    }
    fn reset(&mut self) {
        self.a.reset();
        self.b.reset();
    }
}

/// Logical OR of two signals — see [`SignalExt::or`].
#[derive(Debug)]
pub struct OrSignal<A, B> {
    a: A,
    b: B,
}

impl<A, B> Signal for OrSignal<A, B>
where
    A: Signal,
    B: Signal<Input = A::Input>,
    A::Input: Clone,
{
    type Input = A::Input;
    fn next(&mut self, value: Self::Input) -> Option<SignalEvent> {
        let ea = self.a.next(value.clone());
        let eb = self.b.next(value);
        match (ea, eb) {
            (Some(SignalEvent::Long), _) | (_, Some(SignalEvent::Long)) => Some(SignalEvent::Long),
            (Some(SignalEvent::Short), _) | (_, Some(SignalEvent::Short)) => {
                Some(SignalEvent::Short)
            }
            (Some(SignalEvent::Exit), _) | (_, Some(SignalEvent::Exit)) => Some(SignalEvent::Exit),
            (Some(SignalEvent::Hold), Some(SignalEvent::Hold)) => Some(SignalEvent::Hold),
            (Some(e), None) | (None, Some(e)) => Some(e),
            _ => None,
        }
    }
    fn reset(&mut self) {
        self.a.reset();
        self.b.reset();
    }
}

/// Logical NOT of a signal — see [`SignalExt::not`].
#[derive(Debug)]
pub struct NotSignal<S> {
    inner: S,
}

impl<S: Signal> Signal for NotSignal<S> {
    type Input = S::Input;
    fn next(&mut self, value: Self::Input) -> Option<SignalEvent> {
        self.inner.next(value).map(|e| match e {
            SignalEvent::Long => SignalEvent::Short,
            SignalEvent::Short => SignalEvent::Long,
            other => other,
        })
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_up_fires_only_on_transition() {
        let mut s = CrossUp::new();
        // First bar: warmup, no event.
        assert_eq!(s.next((9.0, 10.0)), None);
        // Still below: hold.
        assert_eq!(s.next((9.5, 10.0)), Some(SignalEvent::Hold));
        // Crosses above: Long.
        assert_eq!(s.next((11.0, 10.0)), Some(SignalEvent::Long));
        // Stays above: hold (no re-fire).
        assert_eq!(s.next((12.0, 10.0)), Some(SignalEvent::Hold));
    }

    #[test]
    fn cross_down_mirrors_cross_up() {
        let mut s = CrossDown::new();
        assert_eq!(s.next((11.0, 10.0)), None);
        assert_eq!(s.next((10.5, 10.0)), Some(SignalEvent::Hold));
        assert_eq!(s.next((9.0, 10.0)), Some(SignalEvent::Short));
        assert_eq!(s.next((8.0, 10.0)), Some(SignalEvent::Hold));
    }

    #[test]
    fn threshold_above_triggers_on_up_crossing() {
        let mut s = ThresholdAbove::new(70.0);
        assert_eq!(s.next(50.0), None);
        assert_eq!(s.next(65.0), Some(SignalEvent::Hold));
        assert_eq!(s.next(75.0), Some(SignalEvent::Long));
        assert_eq!(s.next(80.0), Some(SignalEvent::Hold));
    }

    #[test]
    fn breakout_fires_on_channel_break() {
        let mut s = Breakout::new();
        // (value, upper, lower) — first bar is warmup.
        assert_eq!(s.next((10.0, 11.0, 9.0)), None);
        assert_eq!(s.next((10.5, 11.0, 9.0)), Some(SignalEvent::Hold));
        // value > upper → Long
        assert_eq!(s.next((11.5, 11.0, 9.0)), Some(SignalEvent::Long));
        // After being outside, going back inside → Hold.
        assert_eq!(s.next((10.0, 11.0, 9.0)), Some(SignalEvent::Hold));
    }

    #[test]
    fn and_combinator_requires_agreement() {
        // Both crossing up at the same bar → Long.
        let mut s = CrossUp::new().and(CrossUp::new());
        assert_eq!(s.next((9.0, 10.0)), None);
        assert_eq!(s.next((11.0, 10.0)), Some(SignalEvent::Long));

        // Disagreement → Hold (one fires Long, the other already cooled to Hold).
        let mut a = CrossUp::new();
        let mut b = CrossUp::new();
        // Pre-fire `a` so its second next() is Hold.
        a.next((9.0, 10.0));
        a.next((11.0, 10.0));
        b.next((9.0, 10.0));
        let mut combined = a.and(b);
        // a is past the cross (Hold), b just crosses → AND: Hold.
        assert_eq!(combined.next((11.0, 10.0)), Some(SignalEvent::Hold));
    }

    #[test]
    fn not_combinator_flips_long_short() {
        let mut s = CrossUp::new().not();
        assert_eq!(s.next((9.0, 10.0)), None);
        assert_eq!(s.next((11.0, 10.0)), Some(SignalEvent::Short));
    }
}
