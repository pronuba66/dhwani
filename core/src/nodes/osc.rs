use std::ops::Range;

use crate::{
    Error,
    channel::{ChannelPosition, ChannelPositionsMask},
    event::EventData,
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeResetCtx, NodeTrait},
    port::{PortId, PortProps, PortType},
};

#[derive(Default, Clone)]
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
        match self.mode {
            OscMode::Sin => Box::new(OscSin::new(ctx, self.clone())),
            OscMode::Square => Box::new(OscSquare::new(ctx, self.clone())),
            OscMode::Saw => Box::new(OscSaw::new(ctx, self.clone())),
        }
    }
}

fn build_port_props(is_sin: bool, channel_mask: ChannelPositionsMask) -> Vec<PortProps> {
    if is_sin {
        vec![
            PortProps {
                id: OscProps::PORT_ID_IN_EVENTS,
                kind: PortType::EventsIn,
                auto_connect: true,
                name: "Events",
            },
            PortProps {
                id: OscProps::PORT_ID_PHASE_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Phase",
            },
            PortProps {
                id: OscProps::PORT_ID_W_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Angular frequency",
            },
            PortProps {
                id: OscProps::PORT_ID_MUL_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Multiplier",
            },
            PortProps {
                id: OscProps::PORT_ID_OUTPUT,
                kind: PortType::SignalOut(channel_mask),
                auto_connect: true,
                name: "Output",
            },
        ]
    } else {
        vec![
            PortProps {
                id: OscProps::PORT_ID_IN_EVENTS,
                kind: PortType::EventsIn,
                auto_connect: true,
                name: "Events",
            },
            PortProps {
                id: OscProps::PORT_ID_DUTY_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Duty Cycle",
            },
            PortProps {
                id: OscProps::PORT_ID_PHASE_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Phase",
            },
            PortProps {
                id: OscProps::PORT_ID_W_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Angular frequency",
            },
            PortProps {
                id: OscProps::PORT_ID_MUL_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Multiplier",
            },
            PortProps {
                id: OscProps::PORT_ID_OUTPUT,
                kind: PortType::SignalOut(channel_mask),
                auto_connect: true,
                name: "Output",
            },
        ]
    }
}

struct OscSin {
    chs: Vec<ChannelPosition>,
    props: OscProps,
    port_props: Vec<PortProps>,
    sr: f32,
    phase_delta: f32,
    phase: f32,
}

impl OscSin {
    #[must_use]
    fn new(ctx: &mut NodeCtx, props: OscProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        let port_props = build_port_props(true, props.channel_mask);
        let sr = ctx.sample_rate() as f32;
        let phase_delta = props.w / sr;
        let phase = props.phase;
        Self {
            chs,
            props,
            port_props,
            sr,
            phase_delta,
            phase,
        }
    }

    fn val_static(&self, phase: f32, mul: f32) -> f32 {
        (self.props.phase + phase).sin() * self.props.mul * mul
    }

    fn val(&mut self, phase: f32, w: f32, mul: f32) -> f32 {
        let val = self.val_static(self.phase + phase, mul);
        self.phase += self.phase_delta + w;
        // wrap
        self.phase = self.phase.rem_euclid(std::f32::consts::TAU);
        val
    }
}

impl NodeTrait for OscSin {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let mut output = outputs.get_signals_mut(OscProps::PORT_ID_OUTPUT).unwrap();
        let events = inputs.get_events(OscProps::PORT_ID_IN_EVENTS);
        if let Some(events) = events {
            events.process(step_range, |i, delta_step, event| {
                if let EventData::NoteOn { note, vel } = &event.data {
                    let time = (delta_step as f64 / f64::from(self.sr)) as f32;
                    let val = self.val_static(time * self.props.w * note.mul(), *vel);
                    for &ch in &self.chs {
                        output.get_mut(ch).unwrap()[i] += val;
                    }
                }
            });
        } else {
            let phase = inputs.get_mono(OscProps::PORT_ID_PHASE_CTRL);
            let w = inputs.get_mono(OscProps::PORT_ID_W_CTRL);
            let mul = inputs.get_mono(OscProps::PORT_ID_MUL_CTRL);
            for (i, _) in step_range.enumerate() {
                let phase = phase.as_ref().map_or(0f32, |&phase| phase[i]);
                let w = w.as_ref().map_or(0f32, |&w| w[i]);
                let mul = mul.as_ref().map_or(1f32, |&mul| mul[i]);
                let val = self.val(phase, w, mul);
                for &ch in &self.chs {
                    output.get_mut(ch).unwrap()[i] += val;
                }
            }
        }
    }

    fn reset(&mut self, ctx: &NodeResetCtx) {
        self.phase_delta = self.props.w / ctx.sample_rate as f32;
        // Lets not increment phase from current steps
        // Else the phase will depend also on the modulation signal
        self.phase = self.props.phase;
    }

    fn port_props(&self) -> &[PortProps] {
        &self.port_props
    }

    fn name(&self) -> &'static str {
        "Osc::Sin"
    }
}

struct OscSquare {
    chs: Vec<ChannelPosition>,
    props: OscProps,
    port_props: Vec<PortProps>,
    sr: f32,
    phase_delta: f32,
    phase: f32,
}

impl OscSquare {
    #[must_use]
    fn new(ctx: &mut NodeCtx, props: OscProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        let port_props = build_port_props(false, props.channel_mask);
        let sr = ctx.sample_rate() as f32;
        let phase_delta = props.w / sr;
        let phase = props.phase;
        Self {
            chs,
            props,
            port_props,
            sr,
            phase_delta,
            phase,
        }
    }

    fn val_static(&self, ds: f32, phase: f32, mul: f32) -> f32 {
        let ds = ds + self.props.ds;
        let ds = ds.clamp(0f32, 1f32);
        let val =
            (self.props.phase + phase).rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU;
        if val > ds { 0f32 } else { self.props.mul * mul }
    }

    fn val(&mut self, ds: f32, phase: f32, w: f32, mul: f32) -> f32 {
        let val = self.val_static(ds, self.phase + phase, mul);
        self.phase += self.phase_delta + w;
        // wrap
        self.phase = self.phase.rem_euclid(std::f32::consts::TAU);
        val
    }
}

impl NodeTrait for OscSquare {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let mut output = outputs.get_signals_mut(OscProps::PORT_ID_OUTPUT).unwrap();
        let events = inputs.get_events(OscProps::PORT_ID_IN_EVENTS);
        if let Some(events) = events {
            events.process(step_range, |i, delta_step, event| {
                if let EventData::NoteOn { note, vel } = &event.data {
                    let time = (delta_step as f64 / f64::from(self.sr)) as f32;
                    let val = self.val_static(0f32, time * self.props.w * note.mul(), *vel);
                    for &ch in &self.chs {
                        output.get_mut(ch).unwrap()[i] += val;
                    }
                }
            });
        } else {
            let ds = inputs.get_mono(OscProps::PORT_ID_DUTY_CTRL);
            let w = inputs.get_mono(OscProps::PORT_ID_W_CTRL);
            let phase = inputs.get_mono(OscProps::PORT_ID_PHASE_CTRL);
            let mul = inputs.get_mono(OscProps::PORT_ID_MUL_CTRL);
            for (i, _) in step_range.enumerate() {
                let ds = self.props.ds + ds.as_ref().map_or(0f32, |ds| ds[i]).clamp(0f32, 1f32);
                let phase = self.props.phase + phase.as_ref().map_or(0f32, |phase| phase[i]);
                let w = self.props.w + w.as_ref().map_or(0f32, |w| w[i]);
                let mul = self.props.mul + mul.as_ref().map_or(0f32, |mul| mul[i]);
                let val = self.val(ds, phase, w, mul);
                for &ch in &self.chs {
                    output.get_mut(ch).unwrap()[i] += val;
                }
            }
        }
    }

    fn reset(&mut self, ctx: &NodeResetCtx) {
        self.phase_delta = self.props.w / ctx.sample_rate as f32;
        // Lets not increment phase from current steps
        // Else the phase will depend also on the modulation signal
        self.phase = self.props.phase;
    }

    fn port_props(&self) -> &[PortProps] {
        &self.port_props
    }

    fn name(&self) -> &'static str {
        "Osc::Square"
    }
}

struct OscSaw {
    chs: Vec<ChannelPosition>,
    props: OscProps,
    port_props: Vec<PortProps>,
    sr: f32,
    phase_delta: f32,
    phase: f32,
}

impl OscSaw {
    #[must_use]
    fn new(ctx: &mut NodeCtx, props: OscProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        let port_props = build_port_props(false, props.channel_mask);
        let sr = ctx.sample_rate() as f32;
        let phase_delta = props.w / sr;
        let phase = props.phase;
        Self {
            chs,
            props,
            port_props,
            sr,
            phase_delta,
            phase,
        }
    }

    fn val_static(&self, ds: f32, phase: f32, mul: f32) -> f32 {
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

    fn val(&mut self, ds: f32, phase: f32, w: f32, mul: f32) -> f32 {
        let val = self.val_static(ds, self.phase + phase, mul);
        self.phase += self.phase_delta + w;
        // wrap
        self.phase = self.phase.rem_euclid(std::f32::consts::TAU);
        val
    }
}

impl NodeTrait for OscSaw {
    fn process(
        &mut self,
        step_range: Range<usize>,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let mut output = outputs.get_signals_mut(OscProps::PORT_ID_OUTPUT).unwrap();
        let events = inputs.get_events(OscProps::PORT_ID_IN_EVENTS);
        if let Some(events) = events {
            events.process(step_range, |i, delta_step, event| {
                if let EventData::NoteOn { note, vel } = &event.data {
                    let time = (delta_step as f64 / f64::from(self.sr)) as f32;
                    let val = self.val_static(0f32, time * self.props.w * note.mul(), *vel);
                    for &ch in &self.chs {
                        output.get_mut(ch).unwrap()[i] += val;
                    }
                }
            });
        } else {
            let ds = inputs.get_mono(OscProps::PORT_ID_DUTY_CTRL);
            let phase = inputs.get_mono(OscProps::PORT_ID_PHASE_CTRL);
            let w = inputs.get_mono(OscProps::PORT_ID_W_CTRL);
            let mul = inputs.get_mono(OscProps::PORT_ID_MUL_CTRL);
            for (i, _) in step_range.enumerate() {
                let ds = self.props.ds + ds.as_ref().map_or(0f32, |ds| ds[i]);
                let phase = self.props.phase + phase.as_ref().map_or(0f32, |phase| phase[i]);
                let w = self.props.w + w.as_ref().map_or(0f32, |w| w[i]);
                let mul = self.props.mul + mul.as_ref().map_or(0f32, |mul| mul[i]);
                let val = self.val(ds, phase, w, mul);
                for &ch in &self.chs {
                    output.get_mut(ch).unwrap()[i] += val;
                }
            }
        }
    }

    fn reset(&mut self, ctx: &NodeResetCtx) {
        self.phase_delta = self.props.w / ctx.sample_rate as f32;
        // Lets not increment phase from current steps
        // Else the phase will depend also on the modulation signal
        self.phase = self.props.phase;
    }

    fn port_props(&self) -> &[PortProps] {
        &self.port_props
    }

    fn name(&self) -> &'static str {
        "Osc::Saw"
    }
}
