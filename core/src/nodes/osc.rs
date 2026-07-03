use std::ops::Range;

use crate::{
    Error,
    channel::{ChannelPosition, ChannelPositionsMask},
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeResetCtx, NodeTrait},
    port::{PortId, PortProps, PortType},
    voice::VoiceProcessorTrait,
};

#[derive(Default, Clone, Copy)]
pub enum OscMode {
    #[default]
    Sin,
    Square,
    Saw,
}

#[derive(Default, Clone)]
pub struct OscProps {
    channel_mask: ChannelPositionsMask,
    mode: OscMode,
    // Duty cycle
    ds: f32,
    w: f32, // In rad/s
    phase: f32,
    mul: f32,
}

impl OscProps {
    pub const PORT_ID_IN_EVENTS: PortId = PortId(0);
    pub const PORT_ID_DUTY_CTRL: PortId = PortId(1);
    pub const PORT_ID_PHASE_CTRL: PortId = PortId(2);
    pub const PORT_ID_W_CTRL: PortId = PortId(3);
    pub const PORT_ID_MUL_CTRL: PortId = PortId(4);
    pub const PORT_ID_OUTPUT: PortId = PortId(5);

    /// Create new sine oscialltor builder props
    ///
    /// # Errors
    /// Will return error if parameters are invalid
    #[must_use]
    pub fn new_sin(channel_mask: ChannelPositionsMask, w: f32, mul: f32) -> Result<Self, Error> {
        if w.is_nan() || w.is_infinite() {
            Err(Error::msg("Angular frequency must be finite".into()))
        } else {
            let mul = if mul.is_nan() { 0f32 } else { mul };
            Ok(Self {
                channel_mask,
                mode: OscMode::Sin,
                ds: 0.5f32,
                w,
                phase: 0f32,
                mul,
            })
        }
    }

    /// Create new square oscialltor builder props
    ///
    /// # Errors
    /// Will return error if parameters are invalid
    pub fn new_square(
        channel_mask: ChannelPositionsMask,
        ds: f32,
        w: f32,
        mul: f32,
    ) -> Result<Self, Error> {
        if ds.is_nan() {
            Err(Error::msg("Duty cycle must be a number".into()))
        } else if w.is_nan() || w.is_infinite() {
            Err(Error::msg("Angular frequency must be finite".into()))
        } else {
            let ds = ds.clamp(0f32, 1f32);
            let mul = if mul.is_nan() { 0f32 } else { mul };
            Ok(Self {
                channel_mask,
                mode: OscMode::Square,
                ds,
                w,
                phase: 0f32,
                mul,
            })
        }
    }

    /// Create new saw oscialltor builder props
    ///
    /// # Errors
    /// Will return error if parameters are invalid
    pub fn new_saw(
        channel_mask: ChannelPositionsMask,
        ds: f32,
        w: f32,
        mul: f32,
    ) -> Result<Self, Error> {
        if ds.is_nan() {
            Err(Error::msg("Duty cycle must be a number".into()))
        } else if w.is_nan() || w.is_infinite() {
            Err(Error::msg("Angular frequency must be finite".into()))
        } else {
            let ds = ds.clamp(0f32, 1f32);
            let mul = if mul.is_nan() { 0f32 } else { mul };
            Ok(Self {
                channel_mask,
                mode: OscMode::Saw,
                ds,
                w,
                phase: 0f32,
                mul,
            })
        }
    }
}

impl NodeBuilderTrait for OscProps {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        let port_props = if matches!(self.mode, OscMode::Sin) {
            vec![
                PortProps {
                    id: Self::PORT_ID_IN_EVENTS,
                    kind: PortType::VoicesIn,
                    auto_connect: true,
                    name: "Events",
                },
                PortProps {
                    id: Self::PORT_ID_PHASE_CTRL,
                    kind: PortType::SignalIn,
                    auto_connect: false,
                    name: "Phase",
                },
                PortProps {
                    id: Self::PORT_ID_W_CTRL,
                    kind: PortType::SignalIn,
                    auto_connect: false,
                    name: "Angular frequency",
                },
                PortProps {
                    id: Self::PORT_ID_MUL_CTRL,
                    kind: PortType::SignalIn,
                    auto_connect: false,
                    name: "Multiplier",
                },
                PortProps {
                    id: Self::PORT_ID_OUTPUT,
                    kind: PortType::SignalOut(self.channel_mask),
                    auto_connect: true,
                    name: "Output",
                },
            ]
        } else {
            vec![
                PortProps {
                    id: Self::PORT_ID_IN_EVENTS,
                    kind: PortType::VoicesIn,
                    auto_connect: true,
                    name: "Events",
                },
                PortProps {
                    id: Self::PORT_ID_DUTY_CTRL,
                    kind: PortType::SignalIn,
                    auto_connect: false,
                    name: "Duty Cycle",
                },
                PortProps {
                    id: Self::PORT_ID_PHASE_CTRL,
                    kind: PortType::SignalIn,
                    auto_connect: false,
                    name: "Phase",
                },
                PortProps {
                    id: Self::PORT_ID_W_CTRL,
                    kind: PortType::SignalIn,
                    auto_connect: false,
                    name: "Angular frequency",
                },
                PortProps {
                    id: Self::PORT_ID_MUL_CTRL,
                    kind: PortType::SignalIn,
                    auto_connect: false,
                    name: "Multiplier",
                },
                PortProps {
                    id: Self::PORT_ID_OUTPUT,
                    kind: PortType::SignalOut(self.channel_mask),
                    auto_connect: true,
                    name: "Output",
                },
            ]
        };
        ctx.set_port_props(port_props);
        match self.mode {
            OscMode::Sin => Box::new(Osc::new(ctx, self.clone())),
            OscMode::Square => Box::new(Osc::new(ctx, self.clone())),
            OscMode::Saw => Box::new(Osc::new(ctx, self.clone())),
        }
    }
}

struct SinEventProcessor {
    _w: f32,
    phase: f32,
    mul: f32,
}

impl VoiceProcessorTrait for SinEventProcessor {
    fn process(&self, phase: f32, _w: f32, mul: f32) -> f32 {
        (self.phase + phase).sin() * self.mul * mul
    }
}

struct SquareEventProcessor {
    ds: f32,
    _w: f32,
    phase: f32,
    mul: f32,
}

impl VoiceProcessorTrait for SquareEventProcessor {
    fn process(&self, phase: f32, _w: f32, mul: f32) -> f32 {
        const THRESHOLD: f32 = 1e-6;
        let ds = self.ds.clamp(THRESHOLD, 1f32 - THRESHOLD);
        let ramp = (self.phase + phase).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
        if ramp > ds { 0f32 } else { self.mul * mul }
    }
}

struct SawEventProcessor {
    ds: f32,
    _w: f32,
    phase: f32,
    mul: f32,
}

impl VoiceProcessorTrait for SawEventProcessor {
    fn process(&self, phase: f32, _w: f32, mul: f32) -> f32 {
        const THRESHOLD: f32 = 1e-6;
        let ds = self.ds.clamp(THRESHOLD, 1f32 - THRESHOLD);
        let ramp = (self.phase + phase).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
        let val = if ramp < ds {
            ramp / ds
        } else {
            1f32 - ((ramp - ds) / (1f32 - ds))
        };
        val * self.mul * mul
    }
}

struct Osc {
    sr: f32,
    chs: Vec<ChannelPosition>,
    props: OscProps,
    event_processor: Box<dyn VoiceProcessorTrait>,
    phase_delta: f32,
    phase: f32,
}

impl Osc {
    #[must_use]
    fn new(ctx: &mut NodeCtx, props: OscProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        let phase_delta = props.w / ctx.sample_rate() as f32;
        let phase = props.phase;
        let event_processor: Box<dyn VoiceProcessorTrait> = match props.mode {
            OscMode::Sin => Box::new(SinEventProcessor {
                _w: props.w,
                phase: props.phase,
                mul: props.mul,
            }),
            OscMode::Square => Box::new(SquareEventProcessor {
                ds: props.ds,
                _w: props.w,
                phase: props.phase,
                mul: props.mul,
            }),
            OscMode::Saw => Box::new(SawEventProcessor {
                ds: props.ds,
                _w: props.w,
                phase: props.phase,
                mul: props.mul,
            }),
        };
        Self {
            sr: ctx.sample_rate() as f32,
            chs,
            props,
            event_processor,
            phase_delta,
            phase,
        }
    }

    fn val_sin(&self, phase: f32, mul: f32) -> f32 {
        (self.props.phase + phase).sin() * self.props.mul * mul
    }

    fn val_square(&self, ds: f32, phase: f32, mul: f32) -> f32 {
        let ds = ds + self.props.ds;
        let ds = ds.clamp(0f32, 1f32);
        let val =
            (self.props.phase + phase).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
        if val > ds { 0f32 } else { self.props.mul * mul }
    }

    fn val_saw(&self, ds: f32, phase: f32, mul: f32) -> f32 {
        const THRESHOLD: f32 = 1e-6;
        let ds = ds + self.props.ds;
        let ds = ds.clamp(THRESHOLD, 1f32 - THRESHOLD);
        let ramp =
            (self.props.phase + phase).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
        let val = if ramp < ds {
            ramp / ds
        } else {
            1f32 - ((ramp - ds) / (1f32 - ds))
        };
        val * self.props.mul * mul
    }
}

impl NodeTrait for Osc {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        if self.chs.len() == 0 {
            return;
        }
        let mut output = outputs.get_signals_mut(OscProps::PORT_ID_OUTPUT).unwrap();
        let voices = inputs.get_voices(OscProps::PORT_ID_IN_EVENTS);
        if let Some(voices) = voices {
            voices.process(
                step_range,
                output.get_mut(self.chs[0]).unwrap(),
                self.event_processor.as_ref(),
            );
            for &ch in &self.chs[1..] {
                let _ = output.copy(ch, self.chs[0]);
            }
        } else {
            let ds = inputs.get_mono(OscProps::PORT_ID_DUTY_CTRL);
            let phase = inputs.get_mono(OscProps::PORT_ID_PHASE_CTRL);
            let w = inputs.get_mono(OscProps::PORT_ID_W_CTRL);
            let mul = inputs.get_mono(OscProps::PORT_ID_MUL_CTRL);
            for (i, _) in step_range.enumerate() {
                let ds = ds.as_ref().map_or(0f32, |&ds| ds[i]);
                let phase = phase.as_ref().map_or(0f32, |&phase| phase[i]);
                let w = w.as_ref().map_or(0f32, |&w| w[i]);
                let mul = mul.as_ref().map_or(1f32, |&mul| mul[i]);
                let val = {
                    let val = match self.props.mode {
                        OscMode::Sin => self.val_sin(self.phase + phase, mul),
                        OscMode::Square => self.val_square(ds, self.phase + phase, mul),
                        OscMode::Saw => self.val_saw(ds, self.phase + phase, mul),
                    };
                    self.phase += self.phase_delta + (w / self.sr);
                    // wrap
                    self.phase = self.phase.rem_euclid(std::f32::consts::TAU);
                    val
                };
                output.get_mut(self.chs[0]).unwrap()[i] = val;
            }
            for &ch in &self.chs[1..] {
                let _ = output.copy(ch, self.chs[0]);
            }
        }
    }

    fn reset(&mut self, ctx: &NodeResetCtx) {
        self.phase_delta = self.props.w / ctx.sample_rate as f32;
        // Lets not increment phase from current steps else should also consider
        // that the phase will depend also on the modulation signal. Headache!
        self.phase = self.props.phase;
    }

    fn name(&self) -> &'static str {
        match self.props.mode {
            OscMode::Sin => "Osc::Sin",
            OscMode::Square => "Osc::Square",
            OscMode::Saw => "Osc::Saw",
        }
    }
}
