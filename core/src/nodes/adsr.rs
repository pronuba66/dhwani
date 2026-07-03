use std::ops::Range;

use crate::{
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeResetCtx, NodeTrait},
    port::{PortId, PortProps, PortType},
    time::TimeBaseType,
    voice::{Voice, VoiceEnvelopeProcessorTrait},
};

struct EnvProc {
    a: usize,
    d: usize,
    s: f32,
    r: usize,
}

impl VoiceEnvelopeProcessorTrait for EnvProc {
    fn apply(&self, voice: &Voice, step: usize, val: &mut f32) {
        if step < self.a {
            *val *= step as f32 / self.a as f32
        } else if step < self.a + self.d {
            let r = (step - self.a) as f32 / self.d as f32;
            *val *= (1f32 - r) + self.s * r;
        } else {
            *val *= self.s;
        }
        if let Some(end) = voice.end {
            let duration = end - voice.start;
            if step > duration - self.r {
                let r = (step - (duration - self.r)) as f32 / self.r as f32;
                *val *= 1f32 - r;
            }
        }
    }
}

#[derive(Clone)]
pub struct AdsrProps {
    a: TimeBaseType,
    d: TimeBaseType,
    s: f32,
    r: TimeBaseType,
}

impl AdsrProps {
    pub const PORT_ID_INPUT: PortId = PortId(0);
    pub const PORT_ID_OUTPUT: PortId = PortId(1);

    #[must_use]
    pub const fn new(a: TimeBaseType, d: TimeBaseType, s: f32, r: TimeBaseType) -> Self {
        Self { a, d, s, r }
    }
}

impl NodeBuilderTrait for AdsrProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        let props = self.clone();
        let port_props = vec![
            PortProps {
                id: Self::PORT_ID_INPUT,
                kind: PortType::VoicesIn,
                auto_connect: true,
                name: "Events",
            },
            PortProps {
                id: Self::PORT_ID_OUTPUT,
                kind: PortType::VoicesOut,
                auto_connect: true,
                name: "Events",
            },
        ];
        ctx.set_port_props(port_props);
        let env_proc = EnvProc {
            a: (f64::from(props.a) * f64::from(ctx.sample_rate())) as usize,
            d: (f64::from(props.d) * f64::from(ctx.sample_rate())) as usize,
            s: props.s,
            r: (f64::from(props.r) * f64::from(ctx.sample_rate())) as usize,
        };
        ctx.set_envelope_processors(Self::PORT_ID_OUTPUT, vec![Box::new(env_proc)])
            .unwrap();
        Box::new(Adsr::new(ctx, props))
    }
}

struct Adsr {
    voices: Vec<Voice>,
    // props: AdsrProps,
    addititonal_steps: usize,
}

impl Adsr {
    #[must_use]
    fn new(ctx: &mut NodeCtx, props: AdsrProps) -> Self {
        let voices = Vec::<Voice>::with_capacity(ctx.min_voices_per_frame());
        let addititonal_steps = (f64::from(ctx.sample_rate()) as f64 * f64::from(props.r)) as usize;
        Self {
            voices,
            // props,
            addititonal_steps,
        }
    }
}

impl NodeTrait for Adsr {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let mut output = outputs.get_voices_mut(AdsrProps::PORT_ID_OUTPUT).unwrap();
        // self.voices.clear();
        self.voices.retain(|voice| {
            if let Some(end) = voice.end {
                // end less than that scope should return false
                !(end < step_range.start)
            } else {
                true
            }
        });
        let voices = inputs.get_voices(AdsrProps::PORT_ID_INPUT);
        if let Some(voices) = voices {
            for voice in voices.get() {
                let old = self.voices.iter_mut().rev().find(|v| v.id() == voice.id());
                if let Some(old) = old {
                    if old.end.is_none() {
                        if let Some(end) = voice.end {
                            old.end = Some(end + self.addititonal_steps);
                        }
                    }
                } else {
                    let mut voice = *voice;
                    // Add additional release time to the voice
                    if let Some(end) = voice.end {
                        voice.end = Some(end + self.addititonal_steps);
                    }
                    self.voices.push(voice);
                }
            }
        }
        output.update(step_range, &self.voices);
    }

    fn reset(&mut self, _ctx: &NodeResetCtx) {
        self.voices.clear();
    }

    fn name(&self) -> &'static str {
        "Piano Roll"
    }
}
