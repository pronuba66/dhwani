use std::ops::Range;

use crate::{
    event::{Event, EventData, EventModifierTrait},
    time::{SampleRateBaseType, TimeBaseType},
};

pub struct Adsr {
    a: TimeBaseType,
    d: TimeBaseType,
    s: f32,
    r: TimeBaseType,
}

impl Adsr {
    pub fn new(a: TimeBaseType, d: TimeBaseType, s: f32, r: TimeBaseType) -> Self {
        Self { a, d, s, r }
    }
}

impl EventModifierTrait for Adsr {
    fn should_keep(&self, event: &Event, sr: SampleRateBaseType, step_range: Range<usize>) -> bool {
        let start_time = (step_range.start as f64 / f64::from(sr)) as TimeBaseType;
        event.time.to_seconds(sr) < start_time + self.r
    }

    fn pre(&self, _event: &mut Vec<Event>) {
        todo!()
    }

    fn post(
        &self,
        event: &Event,
        time: f32,
        event_time: f32,
        prev_w_mul: f32,
        prev_mul: f32,
    ) -> (f32, f32) {
        debug_assert!(time >= 0f32, "Invalid time");
        debug_assert!(event_time >= 0f32, "Invalid time");
        match &event.data {
            // Use time for calculating ADSR curve for NoteOn and event_time
            // for NoteOff. NoteOn maybe be invoked multiple times causing adsr
            // to restart everytime for the same event id
            EventData::NoteOn { note, vel } => {
                let mul = if time < self.a {
                    time / self.a
                } else if time < self.a + self.d {
                    let r = ((time - self.a) / self.d) as f32;
                    (1f32 - r) + (r * self.s)
                } else {
                    self.s
                };
                (prev_w_mul * note.mul(), prev_mul * mul * (*vel))
            }
            EventData::NoteOff { note, vel } => {
                let mul = if event_time < self.r {
                    self.s * (1f32 - (event_time / self.r))
                } else {
                    0f32
                };
                (prev_w_mul * note.mul(), prev_mul * mul * (*vel))
            }
        }
    }
}
