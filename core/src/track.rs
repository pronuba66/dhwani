use std::fmt::Debug;

use crate::time::TimeRange;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TrackId(pub(crate) usize);

impl TrackId {
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn val(self) -> usize {
        self.0
    }
}

impl From<usize> for TrackId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct Track {
    id: TrackId,
    time_range: TimeRange,
}

impl Debug for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{track_id: {:?}}}", self.id)
    }
}

impl Track {
    pub(crate) fn new(id: TrackId, time_range: TimeRange) -> Self {
        Self { id, time_range }
    }

    #[must_use]
    pub const fn id(&self) -> TrackId {
        self.id
    }

    #[must_use]
    pub const fn time_range(&self) -> TimeRange {
        self.time_range
    }

    pub fn set_time_range(&mut self, time_range: TimeRange) {
        self.time_range = time_range
    }
}
