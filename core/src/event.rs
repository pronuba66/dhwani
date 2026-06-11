use std::ops::Range;

use crate::{
    midi_note::MidiNote,
    time::{SampleRateBaseType, TimeUnit},
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
pub(crate) struct EventsFrame {
    buffer: Vec<Event>,
    temp_buffer: Vec<Event>,
    ids: Vec<EventId>,
}

impl EventsFrame {
    pub(crate) fn new(min_events_per_frame: usize) -> Self {
        let buffer = Vec::<Event>::with_capacity(min_events_per_frame);
        let temp_buffer = Vec::<Event>::with_capacity(min_events_per_frame);
        let ids = Vec::<EventId>::with_capacity(min_events_per_frame);
        Self {
            buffer,
            temp_buffer,
            ids,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.buffer.clear();
        self.ids.clear();
    }

    fn update(&mut self, sr: SampleRateBaseType, step_range: Range<usize>, events: &[Event]) {
        // Optimize previous events by taking the last event
        self.temp_buffer.clear();
        self.ids.clear();
        for prev_event in self.buffer.iter().rev() {
            if self.ids.contains(&prev_event.id) {
                continue;
            }
            self.ids.push(prev_event.id);
            if !matches!(prev_event.data, EventData::NoteOff) {
                self.temp_buffer.push(*prev_event);
            }
        }
        std::mem::swap(&mut self.temp_buffer, &mut self.buffer);
        for new_event in events {
            if new_event.time.to_samples(sr) >= step_range.end as i64 {
                break;
            }
            if !self.ids.contains(&new_event.id) {
                self.ids.push(new_event.id);
            }
            self.buffer.push(*new_event);
        }
    }

    pub fn process(
        &self,
        sr: SampleRateBaseType,
        step_range: Range<usize>,
        cb: &mut impl FnMut(usize, usize, &Event),
    ) {
        for (i, step) in step_range.enumerate() {
            for &id in &self.ids {
                let last = self.buffer.iter().rev().find(|&event| event.id == id);
                if let Some(event) = last {
                    let step = step as i64;
                    let base_step = event.time.to_samples(sr);
                    if base_step <= step {
                        (cb)(i, (step - base_step) as usize, event);
                    }
                }
            }
        }
    }
}

pub struct EventsMut<'a> {
    sr: SampleRateBaseType,
    frame: &'a mut EventsFrame,
}

impl<'a> EventsMut<'a> {
    pub(crate) const fn new(sr: SampleRateBaseType, frame: &'a mut EventsFrame) -> Self {
        Self { sr, frame }
    }

    /// Update current `self.events` with the live `events`. If the past `end step`
    /// and current `start step` does not match, the previous events are cleared.
    ///
    /// Note: Events may be outside the time range, example, [`EventData::NoteOn`]
    /// maybe be in the past but presenet in the current time range
    ///
    /// # Arguments
    /// * `step_range` - Step range
    /// * `event` - The slice of `Events`
    ///
    /// # Panics
    /// Panics on debug build if `events` not sorted
    ///
    pub fn update(&mut self, step_range: Range<usize>, events: &[Event]) {
        debug_assert!(
            events.is_sorted_by(|a, b| a.time.to_samples(self.sr) <= b.time.to_samples(self.sr)),
            "events not sorted, events: {events:?}"
        );
        self.frame.update(self.sr, step_range, events);
    }
}

pub struct Events<'a> {
    sr: SampleRateBaseType,
    frame: &'a EventsFrame,
}

impl<'a> Events<'a> {
    pub(crate) const fn new(sr: SampleRateBaseType, frame: &'a EventsFrame) -> Self {
        Self { sr, frame }
    }

    /// Process through `time_range` and provide the idx, `TimeUnit` and `Event` through the
    /// callback fn `cb`.
    ///
    /// # Arguments
    /// * `step_range` - Step range
    /// * `cb` - Callback fn with params idx, `Step` and `Event`
    ///
    pub fn process(&self, step_range: Range<usize>, mut cb: impl FnMut(usize, usize, &Event)) {
        self.frame.process(self.sr, step_range, &mut cb);
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
