use std::ops::Range;

use crate::{
    channel::{ChannelPosition, ChannelPositionsMask},
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
};

#[derive(Clone)]
pub struct SimpleMixerProps {
    channel_mask: ChannelPositionsMask,
    muls: Vec<f32>,
}

impl SimpleMixerProps {
    pub const PORT_ID_OUTPUT: PortId = PortId(0);

    #[must_use]
    pub fn new(channel_mask: ChannelPositionsMask, mut muls: Vec<f32>) -> Self {
        for mul in &mut muls {
            if mul.is_nan() {
                *mul = 0f32
            }
        }
        Self { channel_mask, muls }
    }

    #[must_use]
    pub fn get_input_port_id(input_num: usize) -> PortId {
        (1 + input_num).into()
    }
}

impl NodeBuilderTrait for SimpleMixerProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        let mut port_props = Vec::<PortProps>::with_capacity(1 + self.muls.len());
        port_props.push(PortProps {
            id: SimpleMixerProps::PORT_ID_OUTPUT,
            kind: PortType::SignalOut(self.channel_mask),
            auto_connect: true,
            name: "Output",
        });
        for i in 0..self.muls.len() {
            port_props.push(PortProps {
                id: PortId(1 + i),
                kind: PortType::SignalIn,
                auto_connect: true,
                name: "Input",
            });
        }
        ctx.set_port_props(port_props);
        Box::new(SimpleMixer::new(self.clone()))
    }
}

struct SimpleMixer {
    chs: Vec<ChannelPosition>,
    props: SimpleMixerProps,
}

impl SimpleMixer {
    #[must_use]
    fn new(props: SimpleMixerProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        Self { chs, props }
    }
}

impl NodeTrait for SimpleMixer {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let mut output = outputs
            .get_signals_mut(SimpleMixerProps::PORT_ID_OUTPUT)
            .unwrap();
        for i in 0..self.props.muls.len() {
            let port_id = SimpleMixerProps::get_input_port_id(i);
            let input = inputs.get_signals(port_id);
            if let Some(input) = input {
                for &ch in &self.chs {
                    if let Some(signal) = input.get(ch) {
                        for (j, _) in step_range.clone().enumerate() {
                            output.get_mut(ch).unwrap()[j] = signal[j]
                                .mul_add(self.props.muls[i], output.get_mut(ch).unwrap()[j]);
                        }
                    }
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "Simple Mixer"
    }
}
