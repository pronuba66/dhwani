use std::ops::Range;

use crate::{
    event::Event,
    event_modifiers::Adsr,
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
};

#[derive(Clone)]
pub struct PianoRollProps {
    events: Vec<Event>,
}

impl PianoRollProps {
    pub const PORT_ID_OUTPUT: PortId = PortId(0);

    #[must_use]
    pub const fn new(events: Vec<Event>) -> Self {
        Self { events }
    }
}

impl NodeBuilderTrait for PianoRollProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        let mut props = self.clone();
        let sr = ctx.sample_rate();
        props.events.sort_by_key(|event| event.time.to_samples(sr));
        let port_props = vec![PortProps {
            id: Self::PORT_ID_OUTPUT,
            kind: PortType::EventsOut,
            auto_connect: true,
            name: "Events",
        }];
        ctx.set_port_props(port_props);
        ctx.set_event_modifiers(
            PianoRollProps::PORT_ID_OUTPUT,
            vec![Box::new(Adsr::new(0.5f32, 0.25f32, 0.25f32, 0.5f32))],
        )
        .unwrap();
        Box::new(PianoRoll::new(props))
    }
}

struct PianoRoll {
    props: PianoRollProps,
}

impl PianoRoll {
    #[must_use]
    fn new(props: PianoRollProps) -> Self {
        Self { props }
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
            .get_events_mut(PianoRollProps::PORT_ID_OUTPUT)
            .unwrap();
        output.update(step_range, self.props.events.as_slice());
    }

    fn name(&self) -> &'static str {
        "Piano Roll"
    }
}
