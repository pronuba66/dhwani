use crate::{
    channel::ChannelPositionsMask,
    event::Event,
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    nodes::{self, SimpleMixerProps},
    port::{PortId, PortProps, PortProxy, PortType},
    time::ResolvedTimeRange,
};

#[derive(Clone)]
pub struct SimplePianoProps {
    channel_mask: ChannelPositionsMask,
    events: Vec<Event>,
}

impl SimplePianoProps {
    pub const PORT_ID_OUTPUT: PortId = PortId(0);

    #[must_use]
    pub const fn new(channel_mask: ChannelPositionsMask, events: Vec<Event>) -> Self {
        Self {
            channel_mask,
            events,
        }
    }
}

impl NodeBuilderTrait for SimplePianoProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        Box::new(SimplePiano::new(ctx, self.clone()))
    }
}

struct SimplePiano {
    port_props: Vec<PortProps>,
}

impl SimplePiano {
    fn new(ctx: &mut NodeCtx, props: SimplePianoProps) -> Self {
        const PROPS: [(f32, f32); 4] = [
            (220f32, 1f32),    // fundamental
            (440f32, 0.9f32),  // 1st harmonics
            (660f32, 0.75f32), // 2nd harmonics
            (880f32, 0.25f32), // 3rd harmonics
        ];
        let mut port_props = Vec::<PortProps>::with_capacity(1);
        let piano_roll_node_id = ctx
            .add_node(&nodes::PianoRollProps::new(props.events))
            .unwrap();
        let output_node_id = ctx
            .add_node(&nodes::SimpleMixerProps::new(
                props.channel_mask,
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
        for (freq, mul) in &PROPS {
            let id = ctx
                .add_node(&nodes::OscProps::new_sin(props.channel_mask, *freq, *mul))
                .unwrap();
            let _ = ctx.connect_nodes(piano_roll_node_id, id);
            let _ = ctx.connect_nodes(id, output_node_id);
        }
        Self { port_props }
    }
}

impl NodeTrait for SimplePiano {
    fn process(
        &mut self,
        _time_range: ResolvedTimeRange,
        _inputs: &NodeInputs,
        _outputs: &mut NodeOutputs,
    ) {
    }

    fn port_props(&self) -> &[PortProps] {
        &self.port_props
    }

    fn name(&self) -> &'static str {
        "Simple Piano"
    }
}
