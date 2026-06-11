use std::ops::Range;

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz};

use crate::{
    Error,
    channel::{ChannelPosition, ChannelPositionsMask},
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeResetCtx, NodeTrait},
    port::{PortId, PortProps, PortType},
};

#[derive(Clone, Copy)]
pub enum SimpleFilterType {
    LPF, // Low pass filter
    HPF, // High pass filter
    BPF, // Band pass filter
    BSF, // Band stop filter
}

#[derive(Clone)]
pub struct SimpleFilterProps {
    channel_mask: ChannelPositionsMask,
    filter_type: SimpleFilterType,
    freq: f32,
}

impl SimpleFilterProps {
    pub const PORT_ID_INPUT: PortId = PortId(0);
    pub const PORT_ID_OUTPUT: PortId = PortId(1);

    /// Create new low pass filter builder props
    ///
    /// # Errors
    /// Will return error if parameters are invalid
    #[must_use]
    pub fn new_lpf(channel_mask: ChannelPositionsMask, freq: f32) -> Result<Self, Error> {
        if freq.is_nan() || freq.is_infinite() {
            Err(Error::msg("Frequency must be finite".into()))
        } else {
            Ok(Self {
                channel_mask,
                filter_type: SimpleFilterType::LPF,
                freq,
            })
        }
    }

    /// Create new high pass filter builder props
    ///
    /// # Errors
    /// Will return error if parameters are invalid
    #[must_use]
    pub fn new_hpf(channel_mask: ChannelPositionsMask, freq: f32) -> Result<Self, Error> {
        if freq.is_nan() || freq.is_infinite() {
            Err(Error::msg("Frequency must be finite".into()))
        } else {
            Ok(Self {
                channel_mask,
                filter_type: SimpleFilterType::HPF,
                freq,
            })
        }
    }

    /// Create new band pass filter builder props
    ///
    /// # Errors
    /// Will return error if parameters are invalid
    #[must_use]
    pub fn new_bpf(channel_mask: ChannelPositionsMask, freq: f32) -> Result<Self, Error> {
        if freq.is_nan() || freq.is_infinite() {
            Err(Error::msg("Frequency must be finite".into()))
        } else {
            Ok(Self {
                channel_mask,
                filter_type: SimpleFilterType::BPF,
                freq,
            })
        }
    }

    /// Create new band stop filter builder props
    ///
    /// # Errors
    /// Will return error if parameters are invalid
    #[must_use]
    pub fn new_bsf(channel_mask: ChannelPositionsMask, freq: f32) -> Result<Self, Error> {
        if freq.is_nan() || freq.is_infinite() {
            Err(Error::msg("Frequency must be finite".into()))
        } else {
            Ok(Self {
                channel_mask,
                filter_type: SimpleFilterType::BSF,
                freq,
            })
        }
    }
}

impl NodeBuilderTrait for SimpleFilterProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        Box::new(SimpleFilter::new(ctx, self))
    }
}

struct SimpleFilter {
    chs: Vec<ChannelPosition>,
    biquad2: Vec<DirectForm2Transposed<f32>>,
    port_props: Vec<PortProps>,
}

impl SimpleFilter {
    #[must_use]
    pub fn new(ctx: &NodeCtx, props: &SimpleFilterProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        let port_props = vec![
            PortProps {
                id: SimpleFilterProps::PORT_ID_INPUT,
                kind: PortType::SignalIn,
                auto_connect: true,
                name: "Input",
            },
            PortProps {
                id: SimpleFilterProps::PORT_ID_OUTPUT,
                kind: PortType::SignalOut(props.channel_mask),
                auto_connect: true,
                name: "Output",
            },
        ];
        let filter_type = match props.filter_type {
            SimpleFilterType::LPF => biquad::Type::LowPass,
            SimpleFilterType::HPF => biquad::Type::HighPass,
            SimpleFilterType::BPF => biquad::Type::BandPass,
            SimpleFilterType::BSF => biquad::Type::Notch,
        };
        #[allow(clippy::missing_panics_doc)]
        let coeffs = Coefficients::<f32>::from_params(
            filter_type,
            ctx.sample_rate().hz(),
            props.freq.hz(),
            biquad::Q_BUTTERWORTH_F32,
        )
        .unwrap();
        let biquad2 = vec![DirectForm2Transposed::<f32>::new(coeffs); chs.len()];
        Self {
            chs,
            biquad2,
            port_props,
        }
    }
}

impl NodeTrait for SimpleFilter {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let mut output = outputs
            .get_signals_mut(SimpleFilterProps::PORT_ID_OUTPUT)
            .unwrap();
        let input = inputs.get_signals(SimpleFilterProps::PORT_ID_INPUT);
        if let Some(input) = input {
            for (i, &ch) in self.chs.iter().enumerate() {
                if let Some(signal) = input.get(ch) {
                    for (j, _) in step_range.clone().enumerate() {
                        output.get_mut(ch).unwrap()[j] = self.biquad2[i].run(signal[j]);
                    }
                }
            }
        }
    }

    fn reset(&mut self, _ctx: &NodeResetCtx) {
        for biquad2 in &mut self.biquad2 {
            biquad2.reset_state();
        }
    }

    fn port_props(&self) -> &[PortProps] {
        &self.port_props
    }

    fn name(&self) -> &'static str {
        "Simple Filter"
    }
}
