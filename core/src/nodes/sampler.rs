use std::{ops::Range, sync::Arc};

use crate::{
    Error,
    channel::{ChannelPosition, ChannelPositionsMask},
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
};

#[derive(Clone)]
pub struct SampleInfo {
    id: usize,
    n_channels: u16,
    // No resampling implemented
    // sample_rate: SampleRateBaseType,
    n_samples_per_ch: usize,
    data: Arc<Vec<f32>>,
}

impl SampleInfo {
    #[must_use]
    pub const fn new(
        id: usize,
        n_channels: u16,
        // sample_rate: SampleRateBaseType,
        n_samples_per_ch: usize,
        data: Arc<Vec<f32>>,
    ) -> Self {
        Self {
            id,
            n_channels,
            // sample_rate,
            n_samples_per_ch,
            data,
        }
    }

    #[must_use]
    pub const fn id(&self) -> usize {
        self.id
    }

    #[must_use]
    pub const fn n_channels(&self) -> u16 {
        self.n_channels
    }

    // pub fn sample_rate(&self) -> SampleRateBaseType {
    //     self.sample_rate
    // }

    #[must_use]
    pub const fn n_samples_per_ch(&self) -> usize {
        self.n_samples_per_ch
    }

    #[must_use]
    pub fn data(&self) -> Arc<Vec<f32>> {
        self.data.clone()
    }
}

#[derive(Clone)]
pub struct SamplerProps {
    mul: f32,
    info: SampleInfo,
}

impl SamplerProps {
    pub const PORT_ID_MUL_CTRL: PortId = PortId(0);
    pub const PORT_ID_OUTPUT: PortId = PortId(1);

    /// Create new sampler builder props
    /// # Errors
    /// Will return if audio is not mono or stereo
    pub fn new(info: SampleInfo) -> Result<Self, Error> {
        if info.n_channels == 0 || info.n_channels > 2 {
            Err(Error::msg("Sampler only support mono or stereo".into()))
        } else {
            Ok(Self { mul: 1f32, info })
        }
    }
}

impl NodeBuilderTrait for SamplerProps {
    fn build(&self, _ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        Box::new(Sampler::new(self.clone()))
    }
}

struct Sampler {
    chs: Vec<ChannelPosition>,
    props: SamplerProps,
    port_props: Vec<PortProps>,
}

impl Sampler {
    #[must_use]
    pub fn new(props: SamplerProps) -> Self {
        let channel_mask = if props.info.n_channels == 1 {
            ChannelPositionsMask::FRONT_LEFT
        } else if props.info.n_channels == 2 {
            ChannelPositionsMask::FRONT_LEFT | ChannelPositionsMask::FRONT_RIGHT
        } else {
            panic!("Sampler only support mono or stereo")
        };
        let chs = Vec::<ChannelPosition>::from(channel_mask);
        let port_props = vec![
            PortProps {
                id: SamplerProps::PORT_ID_MUL_CTRL,
                kind: PortType::SignalIn,
                auto_connect: true,
                name: "Multiplier",
            },
            PortProps {
                id: SamplerProps::PORT_ID_OUTPUT,
                kind: PortType::SignalOut(channel_mask),
                auto_connect: true,
                name: "Output",
            },
        ];
        Self {
            chs,
            props,
            port_props,
        }
    }
}

impl NodeTrait for Sampler {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let n_channels = usize::from(self.props.info.n_channels);
        let mut output = outputs
            .get_signals_mut(SamplerProps::PORT_ID_OUTPUT)
            .unwrap();
        let mul = inputs.get_mono(SamplerProps::PORT_ID_MUL_CTRL);
        let start = step_range.start;
        let end = self.props.info.n_samples_per_ch.min(step_range.end);
        let len = end.saturating_sub(start);
        for &ch in &self.chs {
            let j = match ch {
                ChannelPosition::FrontLeft => 0usize,
                ChannelPosition::FrontRight => 1usize,
                _ => panic!("Sampler only support mono or stereo"),
            };
            for i in 0..len {
                let mul = self.props.mul + mul.as_ref().map_or(0f32, |mul| mul[i]);
                let val = self.props.info.data[((start + i) * n_channels) + j] * mul;
                output.get_mut(ch).unwrap()[i] += val;
            }
        }
    }

    fn port_props(&self) -> &[PortProps] {
        &self.port_props
    }

    fn name(&self) -> &'static str {
        "Sample"
    }
}
