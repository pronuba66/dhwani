use std::ops::Range;

use crate::{
    midi::{MidiEvent, MidiMsg},
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
    time::{SampleBaseType, SampleRateBaseType},
    voice::Voice,
};

#[derive(Clone)]
pub struct PianoRollProps {
    msgs: Vec<MidiMsg>,
}

impl PianoRollProps {
    pub const PORT_ID_OUTPUT: PortId = PortId(0);

    #[must_use]
    pub fn new(msgs: Vec<MidiMsg>) -> Self {
        Self { msgs }
    }
}

impl NodeBuilderTrait for PianoRollProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        let mut props = self.clone();
        let sr = ctx.sample_rate();
        props.msgs.sort_by_key(|msg| msg.time.to_samples(sr));
        let port_props = vec![PortProps {
            id: Self::PORT_ID_OUTPUT,
            kind: PortType::VoicesOut,
            auto_connect: true,
            name: "Voices",
        }];
        ctx.set_port_props(port_props);
        Box::new(PianoRoll::new(ctx, props))
    }
}

struct PianoRoll {
    sr: SampleRateBaseType,
    voices: Vec<Voice>,
    props: PianoRollProps,
}

impl PianoRoll {
    #[must_use]
    fn new(ctx: &mut NodeCtx, props: PianoRollProps) -> Self {
        let voices = Vec::<Voice>::with_capacity(ctx.min_voices_per_frame());
        Self {
            sr: ctx.sample_rate(),
            props,
            voices,
        }
    }
}

impl NodeTrait for PianoRoll {
    fn process(
        &mut self,
        step_range: Range<usize>,
        _inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let mut output = outputs
            .get_voices_mut(PianoRollProps::PORT_ID_OUTPUT)
            .unwrap();
        // self.voices.clear();
        self.voices.retain(|voice| {
            if let Some(end) = voice.end {
                // end less than that scope should return false
                !(end < step_range.start)
            } else {
                true
            }
        });
        for msg in &self.props.msgs {
            let step = msg.time.to_samples(self.sr);
            if step >= step_range.start as SampleBaseType && step < step_range.end as SampleBaseType
            {
                match &msg.event {
                    MidiEvent::NoteOn { note, vel } => {
                        let voice = self
                            .voices
                            .iter_mut()
                            .rev()
                            .find(|voice| voice.id().0 == msg.id().0);
                        if let Some(voice) = voice {
                            // Replace previous unfinished voice
                            *voice = Voice::new(msg.id().0.into(), step as usize, note.w(), *vel);
                        } else {
                            self.voices.push(Voice::new(
                                msg.id().0.into(),
                                step as usize,
                                note.w(),
                                *vel,
                            ));
                        }
                    }
                    MidiEvent::NoteOff { note: _, vel: _ } => {
                        if let Some(voice) = self
                            .voices
                            .iter_mut()
                            .rev()
                            .find(|voice| voice.id().0 == msg.id().0)
                        {
                            voice.end = Some(step as usize);
                        }
                    }
                }
            }
        }
        output.update(step_range, self.voices.as_slice());
    }

    fn reset(&mut self, _ctx: &crate::node::NodeResetCtx) {
        self.voices.clear();
    }

    fn name(&self) -> &'static str {
        "Piano Roll"
    }
}
