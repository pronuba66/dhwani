use std::ops::Range;

use crate::{
    channel::ChannelPositionsMask,
    midi::MidiMsg,
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    nodes::{self, SimpleMixerProps},
    port::{PortId, PortProps, PortProxy, PortType},
};

#[derive(Clone)]
pub struct SimplePianoProps {
    channel_mask: ChannelPositionsMask,
    msgs: Vec<MidiMsg>,
}

impl SimplePianoProps {
    pub const PORT_ID_OUTPUT: PortId = PortId(0);

    #[must_use]
    pub const fn new(channel_mask: ChannelPositionsMask, msgs: Vec<MidiMsg>) -> Self {
        Self { channel_mask, msgs }
    }
}

impl NodeBuilderTrait for SimplePianoProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        // When Osc is having event input with a given w
        // w becomes the multiplier
        // event.freq * w
        const PROPS: [(f32, f32); 4] = [
            (1f32, 1f32),    // fundamental
            (2f32, 0.9f32),  // 1st harmonics
            (3f32, 0.75f32), // 2nd harmonics
            (4f32, 0.25f32), // 3rd harmonics
        ];
        let mut port_props = Vec::<PortProps>::with_capacity(1);
        let piano_roll_node_id = ctx
            .add_node(&nodes::PianoRollProps::new(self.msgs.clone()))
            .unwrap();
        let adsr_node_id = ctx
            .add_node(&nodes::AdsrProps::new(0.01f32, 0.04f32, 0.9f32, 0.5f32))
            .unwrap();
        ctx.connect_nodes(piano_roll_node_id, adsr_node_id).unwrap();
        let output_node_id = ctx
            .add_node(&nodes::SimpleMixerProps::new(
                self.channel_mask,
                vec![1f32; PROPS.len()],
            ))
            .unwrap();
        let output_node_output_port = ctx
            .get_node(output_node_id)
            .unwrap()
            .get_port(SimpleMixerProps::PORT_ID_OUTPUT)
            .unwrap();
        port_props.push(PortProps {
            id: SimpleMixerProps::PORT_ID_OUTPUT,
            kind: PortType::Proxy(PortProxy::new(0usize.into(), output_node_output_port)),
            auto_connect: true,
            name: "Output",
        });
        for (w, mul) in PROPS {
            let id = ctx
                .add_node(&nodes::OscProps::new_sin(self.channel_mask, w, mul).unwrap())
                .unwrap();
            ctx.connect_nodes(adsr_node_id, id).unwrap();
            ctx.connect_nodes(id, output_node_id).unwrap();
        }
        ctx.set_port_props(port_props);
        Box::new(SimplePiano::default())
    }
}

#[derive(Default)]
struct SimplePiano {}

impl NodeTrait for SimplePiano {
    fn process(
        &mut self,
        _step_range: Range<usize>,
        _inputs: &NodeInputs,
        _outputs: &mut NodeOutputs,
    ) {
    }

    fn name(&self) -> &'static str {
        "Simple Piano"
    }
}
