use crate::{
    midi_note::MidiNote,
    time::{ResolvedTimeRange, SampleBaseType, TimeBaseType, TimeUnit},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct EventId(pub(crate) usize);

impl EventId {
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn val(self) -> usize {
        self.0
    }
}

impl From<usize> for EventId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventData {
    NoteOn { note: MidiNote, vel: f32 },
    NoteOff,
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    id: EventId,
    pub time: TimeUnit,
    pub data: EventData,
}

impl Event {
    #[must_use]
    pub const fn new(id: EventId, time: TimeUnit, data: EventData) -> Self {
        Self { id, time, data }
    }

    #[must_use]
    pub const fn id(&self) -> EventId {
        self.id
    }
}

#[derive(Clone)]
pub struct Events {
    buffer: Vec<Event>,
    temp: Vec<Event>,
    ids: Vec<EventId>,
    last_time: SampleBaseType,
}

impl Events {
    #[must_use]
    pub fn new(min_events_per_frame: usize) -> Self {
        let buffer = Vec::<Event>::with_capacity(min_events_per_frame);
        let temp = Vec::<Event>::with_capacity(min_events_per_frame);
        let ids = Vec::<EventId>::with_capacity(min_events_per_frame);
        Self {
            buffer,
            temp,
            ids,
            last_time: 0,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.ids.clear();
        self.last_time = 0;
    }

    /// Update current `self.events` with the live `events`. If the past `end step`
    /// and current `start step` does not match, the previous events are cleared.
    ///
    /// Note: Events may be outside the time range, example, [`EventData::NoteOn`]
    /// maybe be in the past but presenet in the current time range
    ///
    /// # Arguments
    /// * `time_range` - [`crate::time::ResolvedTimeRange`]
    /// * `event` - The slice of `Events`
    ///
    /// # Panics
    /// Panics on debug build if `events` not sorted
    ///
    pub fn update(&mut self, time_range: ResolvedTimeRange, events: &[Event]) {
        if time_range.start() != self.last_time {
            // Only update if steps are continous
            self.clear();
        }
        debug_assert!(
            events.is_sorted_by(
                |a, b| a.time.to_samples(time_range.sr()) <= b.time.to_samples(time_range.sr())
            ),
            "events not sorted, events: {events:?}"
        );
        // Optimize previous events by taking the last event
        self.temp.clear();
        self.ids.clear();
        for prev_event in self.buffer.iter().rev() {
            if self.ids.contains(&prev_event.id) {
                continue;
            }
            self.ids.push(prev_event.id);
            if !matches!(prev_event.data, EventData::NoteOff) {
                self.temp.push(*prev_event);
            }
        }
        std::mem::swap(&mut self.temp, &mut self.buffer);
        for new_event in events {
            if !self.ids.contains(&new_event.id) {
                self.ids.push(new_event.id);
            }
            self.buffer.push(*new_event);
        }
        self.last_time = time_range.end();
    }

    /// Process through `time_range` and provide the idx, `TimeUnit` and `Event` through the
    /// callback fn `cb`.
    ///
    /// # Arguments
    /// * `time_range` - [`crate::time::ResolvedTimeRange`]
    /// * `cb` - Callback fn with params idx, `Step` and `Event`
    ///
    pub fn process(
        &self,
        time_range: ResolvedTimeRange,
        mut cb: impl FnMut(usize, TimeBaseType, &Event),
    ) {
        let sr = time_range.sr();
        for (i, time) in time_range.into_iter().enumerate() {
            for &id in &self.ids {
                let last = self
                    .buffer
                    .iter()
                    .rev()
                    .find(|&event| event.id == id && event.time.to_seconds(sr) <= time);

                if let Some(event) = last {
                    (cb)(i, time, event);
                }
            }
        }
    }
}

/// Takes in an array of touple of event id, event step, and event data and
/// example usage: events![(0, 0, [`EventData::NoteOn`]), (0, 10, [`EventData::NoteOff`])]
///
#[macro_export]
macro_rules! events {
    ( $( ($id:expr, $step:expr, $data:expr) ),* $(,)? ) => {
        vec![
            $(
                $crate::event::Event::new(
                    $crate::event::EventId::new($id),
                    $step,
                    $data,
                )
            ),*
        ]
    };
}
