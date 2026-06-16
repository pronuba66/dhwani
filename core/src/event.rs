use std::{collections::HashMap, ops::Range};

use crate::{
    midi_note::MidiNote,
    time::{SampleBaseType, SampleRateBaseType, TimeUnit},
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
    NoteOff { note: MidiNote, vel: f32 },
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

pub trait EventModifierTrait {
    // Whether event that is not in bound be kept or not
    fn should_keep(&self, event: &Event, sr: SampleRateBaseType, step_range: Range<usize>) -> bool;
    // Takes event and modify, eg: arpeggiator
    fn pre(&self, event: &mut Vec<Event>);
    fn post(
        &self,
        event: &Event,
        time: f32,
        event_time: f32,
        prev_w_mul: f32,
        prev_mul: f32,
    ) -> (f32, f32);
}

pub trait EventProcessorTrait {
    fn process(&self, time: f32, w_mul: f32, mul: f32) -> f32;
}

pub(crate) struct EventsFrame {
    events: Vec<Event>,
    temp_events: Vec<Event>,
    time_maps: HashMap<EventId, f32>,
    temp_time_maps: HashMap<EventId, f32>,
    modifiers: Vec<Box<dyn EventModifierTrait>>,
}

impl EventsFrame {
    pub(crate) fn new(min_events_per_frame: usize) -> Self {
        let events = Vec::<Event>::with_capacity(min_events_per_frame);
        let temp_events = Vec::<Event>::with_capacity(min_events_per_frame);
        let time_maps = HashMap::<EventId, f32>::with_capacity(min_events_per_frame);
        let temp_time_maps = HashMap::<EventId, f32>::with_capacity(min_events_per_frame);
        Self {
            events,
            temp_events,
            time_maps,
            temp_time_maps,
            modifiers: Vec::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.events.clear();
        self.time_maps.clear();
    }

    pub(crate) fn set_modifiers(&mut self, modifiers: Vec<Box<dyn EventModifierTrait>>) {
        self.modifiers = modifiers;
    }

    fn update(&mut self, sr: SampleRateBaseType, step_range: Range<usize>, events: &[Event]) {
        // Optimize previous events by taking the last event
        self.temp_events.clear();
        self.temp_time_maps.clear();
        // NOTE: events must be sorted
        // Prepend by iterating in reverse order
        for prev_event in self.events.iter().rev() {
            if self.temp_time_maps.contains_key(&prev_event.id) {
                continue;
            }
            // All
            let mut should_keep = false;
            for modifier in &self.modifiers {
                should_keep =
                    should_keep && modifier.should_keep(prev_event, sr, step_range.clone());
            }
            if !should_keep && matches!(prev_event.data, EventData::NoteOff { note: _, vel: _ }) {
                // No need to add event but set the time_maps so that the event
                // wont be checked again
                self.temp_time_maps.insert(prev_event.id, 0f32);
                continue;
            }
            self.temp_events.push(*prev_event);
            let time = self
                .time_maps
                .get(&prev_event.id)
                .map(|&time| time)
                .unwrap_or_else(|| prev_event.time.to_seconds(sr) as f32);
            self.temp_time_maps.insert(prev_event.id, time);
        }
        // Reverse the events to original order
        self.temp_events.reverse();
        std::mem::swap(&mut self.temp_events, &mut self.events);
        std::mem::swap(&mut self.temp_time_maps, &mut self.time_maps);
        // Add new events
        for new_event in events {
            let step = new_event.time.to_samples(sr);
            if step < step_range.start as SampleBaseType || step >= step_range.end as SampleBaseType
            {
                continue;
            }
            self.events.push(*new_event);
            if !self.time_maps.contains_key(&new_event.id) {
                let time = self
                    .time_maps
                    .get(&new_event.id)
                    .map(|&time| time)
                    .unwrap_or_else(|| new_event.time.to_seconds(sr) as f32);
                self.time_maps.insert(new_event.id, time);
            }
        }
        // Takes event and modify, eg: arpeggiator
        // for modifier in &self.modifiers {
        //     modifier.pre(&mut self.events);
        // }
        debug_assert!(
            self.events
                .is_sorted_by(|a, b| a.time.to_samples(sr) <= b.time.to_samples(sr)),
            "events not sorted, events: {:?}",
            self.events,
        );
    }

    pub fn process(
        &self,
        sr: SampleRateBaseType,
        step_range: Range<usize>,
        buffer: &mut [f32],
        processor: &dyn EventProcessorTrait,
    ) {
        for (i, event) in self.events.iter().enumerate() {
            let id = event.id;
            let base_step = event.time.to_samples(sr);
            let start = base_step.max(step_range.start as SampleBaseType) as usize;
            let next_event = self.events[(i + 1)..].iter().find(|event| event.id == id);
            let end = if let Some(event) = next_event {
                event.time.to_samples(sr)
            } else {
                step_range.end as SampleBaseType
            };
            let end = (end as usize).min(step_range.end);
            debug_assert!(end >= start, "Events must be sorted");
            if end <= start {
                continue;
            }
            let base_time = base_step as f32 / sr as f32;
            for step in start..end {
                let curr = step as f32 / sr as f32;
                let time = curr - *self.time_maps.get(&id).unwrap();
                let event_time = curr - base_time;
                let (mut w_mul, mut mul) = (1f32, 1f32);
                for modifier in &self.modifiers {
                    (w_mul, mul) = modifier.post(event, time, event_time, w_mul, mul);
                }
                if mul > 0f32 {
                    buffer[step - step_range.start] = processor.process(time, w_mul, mul);
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

    /// Process through `step_range` with a processor onto the `buffer`
    ///
    /// # Arguments
    /// * `step_range` - Step range
    /// * `cb` - Callback fn with params idx, `Step` and `Event`
    ///

    pub fn process(
        &self,
        step_range: Range<usize>,
        buffer: &mut [f32],
        processor: &dyn EventProcessorTrait,
    ) {
        self.frame.process(self.sr, step_range, buffer, processor);
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
