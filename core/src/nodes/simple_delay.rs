use crate::{
    channel::{ChannelPosition, ChannelPositionsMask},
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
    time::{ResolvedTimeRange, TimeUnit},
};

#[derive(Clone)]
pub struct SimpleDelayProps {
    channel_mask: ChannelPositionsMask,
    delay: TimeUnit,
    mul: f32,
}

impl SimpleDelayProps {
    pub const PORT_ID_INPUT: PortId = PortId(0);
    pub const PORT_ID_OUTPUT: PortId = PortId(1);

    #[must_use]
    pub const fn new(channel_mask: ChannelPositionsMask, delay: TimeUnit, mul: f32) -> Self {
        Self {
            channel_mask,
            delay,
            mul,
        }
    }
}

impl NodeBuilderTrait for SimpleDelayProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        Box::new(SimpleDelay::new(ctx, self.clone()))
    }
}

struct SimpleDelay {
    chs: Vec<ChannelPosition>,
    buffer_size: usize,
    buffers: Vec<Vec<f32>>,
    props: SimpleDelayProps,
    port_props: Vec<PortProps>,
}

impl SimpleDelay {
    #[must_use]
    fn new(ctx: &NodeCtx, props: SimpleDelayProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        let port_props = vec![
            PortProps {
                id: SimpleDelayProps::PORT_ID_INPUT,
                kind: PortType::SignalIn,
                auto_connect: true,
                name: "Input",
            },
            PortProps {
                id: SimpleDelayProps::PORT_ID_OUTPUT,
                kind: PortType::SignalOut(props.channel_mask),
                auto_connect: true,
                name: "Output",
            },
        ];
        let buffer_size = ctx.buffer_size() + props.delay.to_samples(ctx.sample_rate()) as usize;
        let mut buffers = Vec::<Vec<f32>>::with_capacity(chs.len());
        for _ in 0..chs.len() {
            buffers.push(vec![0f32; buffer_size]);
        }
        Self {
            chs,
            buffer_size,
            buffers,
            props,
            port_props,
        }
    }
}

impl NodeTrait for SimpleDelay {
    fn process(
        &mut self,
        time_range: ResolvedTimeRange,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let mut output = outputs
            .get_signals_mut(SimpleDelayProps::PORT_ID_OUTPUT)
            .unwrap();
        for (i, &ch) in self.chs.iter().enumerate() {
            for (j, _) in time_range.into_iter().enumerate() {
                output.get_mut(ch).unwrap()[j] += self.buffers[i][j];
            }
            // Shift buffer
            self.buffers[i].copy_within(time_range.duraiton().., 0usize);
        }
        let old = self.buffer_size - time_range.duraiton();
        let input = inputs.get_signals(SimpleDelayProps::PORT_ID_INPUT);
        if let Some(input) = input {
            for (i, &ch) in self.chs.iter().enumerate() {
                if let Some(signal) = input.get(ch) {
                    let stop = old + signal.len();
                    for (j, val) in &mut self.buffers[i][old..stop].iter_mut().enumerate() {
                        *val = signal[j] * self.props.mul;
                    }
                    self.buffers[i][stop..].fill(0f32);
                } else {
                    self.buffers[i][old..].fill(0f32);
                }
            }
        } else {
            for i in 0..self.chs.len() {
                self.buffers[i][old..].fill(0f32);
            }
        }
    }

    fn port_props(&self) -> &[PortProps] {
        &self.port_props
    }

    fn name(&self) -> &'static str {
        "Simple Delay"
    }
}
