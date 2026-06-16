use std::ops::Range;

use crate::{
    Error,
    channel::{ChannelPosition, ChannelPositionsMask},
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
    time::TimeUnit,
};

#[derive(Clone)]
pub struct DelayProps {
    channel_mask: ChannelPositionsMask,
    delay: TimeUnit,
    mul: f32,
}

impl DelayProps {
    pub const PORT_ID_INPUT: PortId = PortId(0);
    pub const PORT_ID_OUTPUT: PortId = PortId(1);

    /// Create new simple delay builder props
    ///
    /// # Errors
    /// Will return error if delay is negative
    pub fn new(
        channel_mask: ChannelPositionsMask,
        delay: TimeUnit,
        mul: f32,
    ) -> Result<Self, Error> {
        if !delay.is_non_negative_finite() {
            return Err(Error::msg("Delat must be non negative".into()));
        } else {
            let mul = if mul.is_nan() { 0f32 } else { mul };
            Ok(Self {
                channel_mask,
                delay,
                mul,
            })
        }
    }
}

impl NodeBuilderTrait for DelayProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        let port_props = vec![
            PortProps {
                id: DelayProps::PORT_ID_INPUT,
                kind: PortType::SignalIn,
                auto_connect: true,
                name: "Input",
            },
            PortProps {
                id: DelayProps::PORT_ID_OUTPUT,
                kind: PortType::SignalOut(self.channel_mask),
                auto_connect: true,
                name: "Output",
            },
        ];
        ctx.set_port_props(port_props);
        Box::new(Delay::new(ctx, self.clone()))
    }
}

struct Delay {
    chs: Vec<ChannelPosition>,
    buffer_size: usize,
    buffers: Vec<Vec<f32>>,
    props: DelayProps,
}

impl Delay {
    #[must_use]
    fn new(ctx: &NodeCtx, props: DelayProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        // Limit additional size to 64 seconds
        let additional_size = props
            .delay
            .to_samples(ctx.sample_rate())
            .min(TimeUnit::Seconds(64f32).to_samples(ctx.sample_rate()))
            as usize;
        let buffer_size = ctx.buffer_size() + additional_size;
        let mut buffers = Vec::<Vec<f32>>::with_capacity(chs.len());
        for _ in 0..chs.len() {
            buffers.push(vec![0f32; buffer_size]);
        }
        Self {
            chs,
            buffer_size,
            buffers,
            props,
        }
    }
}

impl NodeTrait for Delay {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let duration = step_range.end - step_range.start;
        let mut output = outputs.get_signals_mut(DelayProps::PORT_ID_OUTPUT).unwrap();
        for (i, &ch) in self.chs.iter().enumerate() {
            for (j, _) in step_range.clone().enumerate() {
                output.get_mut(ch).unwrap()[j] += self.buffers[i][j];
            }
            // Shift buffer
            self.buffers[i].copy_within(duration.., 0usize);
        }
        let old = self.buffer_size - duration;
        let input = inputs.get_signals(DelayProps::PORT_ID_INPUT);
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

    fn name(&self) -> &'static str {
        "Simple Delay"
    }
}
