use crate::{
    event::Event,
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
    time::ResolvedTimeRange,
};

#[derive(Clone)]
pub struct PianoRollProps {
    events: Vec<Event>,
}

impl PianoRollProps {
    pub const PORT_ID_OUTPUT: PortId = PortId(0);

    const PORT_PROPS: [PortProps; 1] = [PortProps {
        id: Self::PORT_ID_OUTPUT,
        kind: PortType::EventsOut,
        auto_connect: true,
        name: "Events",
    }];

    #[must_use]
    pub const fn new(events: Vec<Event>) -> Self {
        Self { events }
    }
}

impl NodeBuilderTrait for PianoRollProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        let mut props = self.clone();
        let sr = ctx.sample_rate();
        props
            .events
            .sort_by(|a, b| a.time.to_samples(sr).cmp(&b.time.to_samples(sr)));
        Box::new(PianoRoll::new(props))
    }
}

struct PianoRoll {
    props: PianoRollProps,
}

impl PianoRoll {
    #[must_use]
    const fn new(props: PianoRollProps) -> Self {
        Self { props }
    }
}

impl NodeTrait for PianoRoll {
    fn process(
        &mut self,
        time_range: ResolvedTimeRange,
        _inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let output_events = outputs
            .get_events_mut(PianoRollProps::PORT_ID_OUTPUT)
            .unwrap();
        output_events.update(time_range, self.props.events.as_slice());
    }

    fn port_props(&self) -> &[PortProps] {
        &PianoRollProps::PORT_PROPS
    }

    fn name(&self) -> &'static str {
        "Piano Roll"
    }
}
