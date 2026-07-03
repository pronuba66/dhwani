use std::ops::Range;

use crate::time::SampleRateBaseType;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct VoiceId(pub(crate) usize);

impl VoiceId {
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn val(self) -> usize {
        self.0
    }
}

impl From<usize> for VoiceId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Voice {
    id: VoiceId,
    pub start: usize,
    pub end: Option<usize>,
    pub w: f32,
    pub vel: f32,
    pub phase: f32,
}

impl Voice {
    #[must_use]
    pub const fn new(id: VoiceId, start: usize, w: f32, vel: f32) -> Self {
        Self {
            id,
            start,
            end: None,
            w,
            vel,
            phase: 0f32,
        }
    }

    #[must_use]
    pub const fn id(&self) -> VoiceId {
        self.id
    }
}

pub trait VoiceProcessorTrait {
    fn process(&self, phase: f32, w: f32, mul: f32) -> f32;
}

pub trait VoiceEnvelopeProcessorTrait {
    fn apply(&self, voice: &Voice, step: usize, val: &mut f32);
}

pub(crate) struct VoicesFrame {
    sr: SampleRateBaseType,
    voices: Vec<Voice>,
    env_procs: Vec<Box<dyn VoiceEnvelopeProcessorTrait>>,
}

impl VoicesFrame {
    pub(crate) fn new(sr: SampleRateBaseType, min_voices_per_frame: usize) -> Self {
        let voices = Vec::<Voice>::with_capacity(min_voices_per_frame);
        Self {
            sr,
            voices,
            env_procs: vec![],
        }
    }

    pub(crate) fn set_env_procs(&mut self, procs: Vec<Box<dyn VoiceEnvelopeProcessorTrait>>) {
        self.env_procs = procs;
    }

    pub(crate) fn clear(&mut self) {
        self.voices.clear();
    }

    fn update(&mut self, step_range: Range<usize>, voices: &[Voice]) {
        // Add new voice
        self.voices.clear();
        for new_voice in voices {
            if new_voice.start >= step_range.end {
                continue;
            }
            if let Some(end) = new_voice.end
                && end < step_range.start
            {
                continue;
            }
            self.voices.push(*new_voice);
        }
        // Voice need not be sorted like events
        // self.voices.sort_by_key(|voice| voice.start);
    }

    pub fn process(
        &self,
        step_range: Range<usize>,
        buffer: &mut [f32],
        processor: &dyn VoiceProcessorTrait,
    ) {
        for voice in &self.voices {
            let base_step = voice.start;
            let start = base_step.max(step_range.start);
            let end = voice.end.unwrap_or(step_range.end).min(step_range.end);
            if start >= end {
                // This can be true when end is None and
                // voice.start is beyond the current scope
                continue;
            }
            let w = voice.w;
            let mul = voice.vel;
            let mut phase =
                (w * (start - base_step) as f32 / self.sr as f32).rem_euclid(std::f32::consts::TAU);
            let phase_delta = w / self.sr as f32;
            for step in start..end {
                let mut f = processor.process(phase, w, mul);
                for proc in &self.env_procs {
                    proc.apply(voice, step - base_step, &mut f);
                }
                buffer[step - step_range.start] += f;
                phase = (phase + phase_delta).rem_euclid(std::f32::consts::TAU);
            }
        }
    }
}

pub struct VoicesMut<'a> {
    frame: &'a mut VoicesFrame,
}

impl<'a> VoicesMut<'a> {
    pub(crate) const fn new(frame: &'a mut VoicesFrame) -> Self {
        Self { frame }
    }

    /// Update current `self.events` with the live `events`.
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
    pub fn update(&mut self, step_range: Range<usize>, voices: &[Voice]) {
        debug_assert!(
            voices.is_sorted_by_key(|voice| voice.start),
            "voices not sorted, voices: {voices:?}"
        );
        self.frame.update(step_range, voices);
    }

    #[must_use]
    pub fn get(&self) -> &[Voice] {
        &self.frame.voices
    }

    #[must_use]
    pub fn get_mut(&mut self) -> &mut [Voice] {
        &mut self.frame.voices
    }
}

pub struct Voices<'a> {
    frame: &'a VoicesFrame,
}

impl<'a> Voices<'a> {
    pub(crate) const fn new(frame: &'a VoicesFrame) -> Self {
        Self { frame }
    }

    /// Process through `step_range` with a processor onto the `buffer`
    ///
    /// # Arguments
    /// * `step_range` - Step range
    /// * `cb` - Callback fn with params idx, `Step` and `Voice`
    ///

    pub fn process(
        &self,
        step_range: Range<usize>,
        buffer: &mut [f32],
        processor: &dyn VoiceProcessorTrait,
    ) {
        self.frame.process(step_range, buffer, processor);
    }

    #[must_use]
    pub fn get(&self) -> &[Voice] {
        &self.frame.voices
    }
}
