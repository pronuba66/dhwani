use std::fmt::Debug;

/// Time in seconds base type
pub type TimeBaseType = f64;
/// Sample base type
pub type SampleBaseType = i64;
/// Sample rate base type
pub type SampleRateBaseType = u32;
/// Step base type
pub type StepBaseType = i64;
/// BPM base type
pub type BPMBaseType = f32;

#[derive(Clone, Copy)]
pub struct TimeSignature {
    num: u8,
    den: u8,
}

impl Debug for TimeSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.num, self.den)
    }
}

impl TimeSignature {
    /// Create time signature of the form {num} / {den}
    ///
    #[must_use]
    pub const fn new(num: u8, den: u8) -> Self {
        Self { num, den }
    }
}

#[derive(Clone, Copy)]
pub enum TimeUnit {
    Seconds(TimeBaseType),
    Samples(SampleBaseType),
    Steps {
        steps: StepBaseType,
        ts: TimeSignature,
        bpm: BPMBaseType,
    },
}

impl From<TimeBaseType> for TimeUnit {
    fn from(value: TimeBaseType) -> Self {
        Self::Seconds(value)
    }
}

impl From<SampleBaseType> for TimeUnit {
    fn from(value: SampleBaseType) -> Self {
        Self::Samples(value)
    }
}

impl Default for TimeUnit {
    fn default() -> Self {
        Self::Samples(0)
    }
}

impl Debug for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seconds(seconds) => write!(f, "{}", *seconds),
            Self::Samples(samples) => write!(f, "{}", *samples),
            Self::Steps { steps, ts, bpm } => write!(f, "{}[{:?}]@{:.2}bpm", *steps, ts, *bpm),
        }
    }
}

impl TimeUnit {
    #[must_use]
    pub const fn is_non_negative_finite(&self) -> bool {
        match self {
            Self::Seconds(seconds) => seconds.is_sign_positive(),
            Self::Samples(samples) => *samples >= 0,
            Self::Steps {
                steps,
                ts: _,
                bpm: _,
            } => *steps >= 0,
        }
    }

    #[must_use]
    pub fn to_seconds(self, sr: SampleRateBaseType) -> TimeBaseType {
        match self {
            Self::Seconds(seconds) => seconds,
            Self::Samples(samples) => samples as f64 / f64::from(sr),
            Self::Steps { steps, ts, bpm } => {
                // steps * (60 / bpm) * (4 / den)
                let steps = steps as f64;
                let seconds_per_beat = (4f64 * (60f64 / f64::from(bpm))) / f64::from(ts.den); // BPM is in quater notes per minute
                steps * seconds_per_beat
            }
        }
    }

    #[must_use]
    pub fn to_samples(self, sr: SampleRateBaseType) -> SampleBaseType {
        match self {
            Self::Seconds(seconds) => (seconds * f64::from(sr)).round() as i64,
            Self::Samples(samples) => samples,
            Self::Steps {
                steps: _,
                ts: _,
                bpm: _,
            } => Self::Seconds(self.to_seconds(sr)).to_samples(sr),
        }
    }
}

/// Describes a point in time relative to a reference position on the timeline.
///
/// # Variants
///
/// - [`TimeFrom::Start`]   — offset from the beginning of the arrangement.
/// - [`TimeFrom::End`]     — offset backwards from the end of a region or clip.
/// - [`TimeFrom::Current`] — offset from the current playhead position.
///
#[derive(Debug, Clone, Copy)]
pub enum TimeFrom {
    /// Offset (in beats) measured from the arrangement start (t = 0).
    Start(TimeUnit),
    /// Offset (in beats) measured backwards from the end of a region.
    End(TimeUnit),
    /// Offset (in beats) measured forward from the current playhead.
    Current(TimeUnit),
}

#[derive(Clone, Copy)]
pub struct TimeRange {
    start: TimeUnit,
    // None value means ticks to infinity
    end: Option<TimeUnit>,
}

impl Default for TimeRange {
    fn default() -> Self {
        Self {
            start: TimeUnit::Samples(0),
            end: None,
        }
    }
}

impl Debug for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}..{:?}", self.start, self.end)
    }
}

impl TimeRange {
    #[must_use]
    pub const fn new(start: TimeUnit, end: Option<TimeUnit>) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn start(&self) -> TimeUnit {
        self.start
    }

    #[must_use]
    pub const fn end(&self) -> Option<TimeUnit> {
        self.end
    }
}
