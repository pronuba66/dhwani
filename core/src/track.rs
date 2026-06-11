use std::fmt::Debug;

use crate::time::{SampleBaseType, TimeRange};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TrackId(pub usize);

impl From<usize> for TrackId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct Track {
    id: TrackId,
    time_range: TimeRange,
    start_step: SampleBaseType,
    end_step: Option<SampleBaseType>,
}

impl Debug for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{track_id: {:?}}}", self.id)
    }
}

impl Track {
    pub(crate) const fn new(id: TrackId, time_range: TimeRange) -> Self {
        Self {
            id,
            time_range,
            start_step: 0,
            end_step: None,
        }
    }

    #[must_use]
    pub const fn id(&self) -> TrackId {
        self.id
    }

    #[must_use]
    pub const fn start_step(&self) -> SampleBaseType {
        self.start_step
    }

    pub const fn set_start_step(&mut self, start_step: SampleBaseType) {
        self.start_step = start_step
    }

    #[must_use]
    pub const fn end_step(&self) -> Option<SampleBaseType> {
        self.end_step
    }

    pub const fn set_end_step(&mut self, end_step: Option<SampleBaseType>) {
        self.end_step = end_step
    }

    #[must_use]
    pub const fn time_range(&self) -> TimeRange {
        self.time_range
    }

    pub const fn set_time_range(&mut self, time_range: TimeRange) {
        self.time_range = time_range;
    }
}
