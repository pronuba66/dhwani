use std::ops::Range;

use crate::{
    channel::{ChannelPosition, ChannelPositionsMask},
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
};

#[derive(Default, Clone)]
pub struct StereoProps {}

impl StereoProps {
    pub const PORT_ID_INPUT: PortId = PortId(0);
    pub const PORT_ID_OUTPUT: PortId = PortId(1);

    const PORT_PROPS: [PortProps; 2] = [
        PortProps {
            id: Self::PORT_ID_INPUT,
            kind: PortType::SignalIn,
            auto_connect: true,
            name: "Input",
        },
        PortProps {
            id: Self::PORT_ID_OUTPUT,
            kind: PortType::SignalOut(
                ChannelPositionsMask::from_bits(
                    ChannelPositionsMask::FRONT_LEFT.bits()
                        | ChannelPositionsMask::FRONT_RIGHT.bits(),
                )
                .unwrap(),
            ),
            auto_connect: true,
            name: "Output",
        },
    ];
}

impl NodeBuilderTrait for StereoProps {
    fn build(&self, _ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        Box::new(Stereo::default())
    }
}

#[derive(Default)]
struct Stereo {}

impl NodeTrait for Stereo {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let mut output = outputs
            .get_signals_mut(StereoProps::PORT_ID_OUTPUT)
            .unwrap();
        let input = inputs.get_signals(StereoProps::PORT_ID_INPUT);
        if let Some(input) = input
            && let Some(signal) = input.get(ChannelPosition::FrontLeft)
        {
            for (i, _) in step_range.enumerate() {
                output.get_mut(ChannelPosition::FrontLeft).unwrap()[i] += signal[i];
                output.get_mut(ChannelPosition::FrontRight).unwrap()[i] += signal[i];
            }
        }
    }

    fn port_props(&self) -> &[PortProps] {
        &StereoProps::PORT_PROPS
    }

    fn name(&self) -> &'static str {
        "Stero"
    }
}
