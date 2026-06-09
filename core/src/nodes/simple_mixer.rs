use crate::{
    channel::{ChannelPosition, ChannelPositionsMask},
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
    time::ResolvedTimeRange,
};

#[derive(Clone)]
pub struct SimpleMixerProps {
    channel_mask: ChannelPositionsMask,
    muls: Vec<f32>,
}

impl SimpleMixerProps {
    pub const PORT_ID_OUTPUT: PortId = PortId(0);

    #[must_use]
    pub fn new(channel_mask: ChannelPositionsMask, muls: Vec<f32>) -> Self {
        Self { channel_mask, muls }
    }

    #[must_use]
    pub fn get_input_port_id(input_num: usize) -> PortId {
        (1 + input_num).into()
    }
}

impl NodeBuilderTrait for SimpleMixerProps {
    fn build(&self, _ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        Box::new(SimpleMixer::new(self.clone()))
    }
}

struct SimpleMixer {
    chs: Vec<ChannelPosition>,
    props: SimpleMixerProps,
    port_props: Vec<PortProps>,
}

impl SimpleMixer {
    #[must_use]
    fn new(props: SimpleMixerProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        let mut port_props = Vec::<PortProps>::with_capacity(1 + props.muls.len());
        port_props.push(PortProps {
            id: SimpleMixerProps::PORT_ID_OUTPUT,
            kind: PortType::SignalOut(props.channel_mask),
            auto_connect: true,
            name: "Output",
        });
        for i in 0..props.muls.len() {
            port_props.push(PortProps {
                id: PortId(1 + i),
                kind: PortType::SignalIn,
                auto_connect: true,
                name: "Input",
            });
        }
        Self {
            chs,
            props,
            port_props,
        }
    }
}

impl NodeTrait for SimpleMixer {
    fn process(
        &mut self,
        time_range: ResolvedTimeRange,
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
                        for (j, _) in time_range.into_iter().enumerate() {
                            output.get_mut(ch).unwrap()[j] += signal[j] * self.props.muls[i];
                        }
                    }
                }
            }
        }
    }

    fn port_props(&self) -> &[PortProps] {
        &self.port_props
    }

    fn name(&self) -> &'static str {
        "Simple Mixer"
    }
}
