use std::ops::Range;

use crate::{
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
    voice::Voice,
};

#[derive(Clone)]
pub struct TransposeProps {
    cents: f32,
}

impl TransposeProps {
    pub const PORT_ID_INPUT: PortId = PortId(0);
    pub const PORT_ID_OUTPUT: PortId = PortId(1);

    #[must_use]
    pub const fn new(cents: f32) -> Self {
        Self { cents }
    }
}

impl NodeBuilderTrait for TransposeProps {
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
        Box::new(Transpose::new(ctx, props))
    }
}

struct Transpose {
    voices: Vec<Voice>,
    // props: TransposeProps,
    w_mul: f32,
}

impl Transpose {
    #[must_use]
    fn new(ctx: &mut NodeCtx, props: TransposeProps) -> Self {
        let voices = Vec::<Voice>::with_capacity(ctx.min_voices_per_frame());
        let w_mul = 2.0_f32.powf(props.cents / 1200f32);
        Self {
            voices,
            // props,
            w_mul,
        }
    }
}

impl NodeTrait for Transpose {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let mut output = outputs
            .get_voices_mut(TransposeProps::PORT_ID_OUTPUT)
            .unwrap();
        self.voices.clear();
        let voices = inputs.get_voices(TransposeProps::PORT_ID_INPUT);
        if let Some(voices) = voices {
            for voice in voices.get() {
                let mut voice = *voice;
                voice.w = voice.w * self.w_mul;
                self.voices.push(voice);
            }
        }
        output.update(step_range, &self.voices);
    }

    fn name(&self) -> &'static str {
        "Piano Roll"
    }
}
