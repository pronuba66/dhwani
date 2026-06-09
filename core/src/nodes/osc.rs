use crate::{
    Error,
    channel::{ChannelPosition, ChannelPositionsMask},
    event::EventData,
    node::{NodeBuilderTrait, NodeCtx, NodeInputs, NodeOutputs, NodeTrait},
    port::{PortId, PortProps, PortType},
    time::ResolvedTimeRange,
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
    freq: f32,
    phase: f32,
    mul: f32,
}

impl OscProps {
    pub const PORT_ID_IN_EVENTS: PortId = PortId(0);
    pub const PORT_ID_DUTY_CTRL: PortId = PortId(1);
    pub const PORT_ID_FREQ_CTRL: PortId = PortId(2);
    pub const PORT_ID_PHASE_CTRL: PortId = PortId(3);
    pub const PORT_ID_MUL_CTRL: PortId = PortId(4);
    pub const PORT_ID_OUTPUT: PortId = PortId(5);

    #[must_use]
    pub const fn new_sin(channel_mask: ChannelPositionsMask, freq: f32, mul: f32) -> Self {
        Self {
            channel_mask,
            mode: OscMode::Sin,
            ds: 0.5f32,
            freq,
            phase: 0f32,
            mul,
        }
    }

    ///
    /// # Errors
    /// Return `Err` if duty cycle `ds` is invalid
    ///
    pub fn new_square(
        channel_mask: ChannelPositionsMask,
        ds: f32,
        freq: f32,
        mul: f32,
    ) -> Result<Self, Error> {
        if (0f32..=1f32).contains(&ds) {
            Ok(Self {
                channel_mask,
                mode: OscMode::Square,
                ds,
                freq,
                phase: 0f32,
                mul,
            })
        } else {
            Err(Error::msg("Duty cycle must be [0, 1]".into()))
        }
    }

    ///
    /// # Errors
    /// Return `Err` if duty cycle `ds` is invalid
    ///
    pub fn new_saw(
        channel_mask: ChannelPositionsMask,
        ds: f32,
        freq: f32,
        mul: f32,
    ) -> Result<Self, Error> {
        if (0f32..=1f32).contains(&ds) {
            Ok(Self {
                channel_mask,
                mode: OscMode::Saw,
                ds,
                freq,
                phase: 0f32,
                mul,
            })
        } else {
            Err(Error::msg("Duty cycle must be [0, 1]".into()))
        }
    }
}

impl NodeBuilderTrait for OscProps {
    fn build(&self, _ctx: &mut NodeCtx) -> Box<dyn NodeTrait> {
        match self.mode {
            OscMode::Sin => Box::new(OscSin::new(self.clone())),
            OscMode::Square => Box::new(OscSquare::new(self.clone())),
            OscMode::Saw => Box::new(OscSaw::new(self.clone())),
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
                id: OscProps::PORT_ID_FREQ_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Frequency",
            },
            PortProps {
                id: OscProps::PORT_ID_PHASE_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Phase",
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
                id: OscProps::PORT_ID_FREQ_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Frequency",
            },
            PortProps {
                id: OscProps::PORT_ID_PHASE_CTRL,
                kind: PortType::SignalIn,
                auto_connect: false,
                name: "Phase",
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
}

impl OscSin {
    #[must_use]
    fn new(props: OscProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        let port_props = build_port_props(true, props.channel_mask);
        Self {
            chs,
            props,
            port_props,
        }
    }

    fn val(time: f32, freq: f32, phase: f32, mul: f32) -> f32 {
        (time * freq).mul_add(std::f32::consts::TAU, phase).sin() * mul
    }
}

impl NodeTrait for OscSin {
    fn process(
        &mut self,
        time_range: ResolvedTimeRange,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let sr = time_range.sr();
        let mut output = outputs.get_signals_mut(OscProps::PORT_ID_OUTPUT).unwrap();
        let events = inputs.get_events(OscProps::PORT_ID_IN_EVENTS);
        if let Some(events) = events {
            events.process(time_range, |i, time, event| {
                if let EventData::NoteOn { note, vel } = &event.data {
                    let val = Self::val(
                        (time - event.time.to_seconds(sr)) as f32,
                        self.props.freq * note.mul(),
                        self.props.phase,
                        self.props.mul * vel,
                    );
                    for &ch in &self.chs {
                        output.get_mut(ch).unwrap()[i] += val;
                    }
                }
            });
        } else {
            let freq = inputs.get_mono(OscProps::PORT_ID_FREQ_CTRL);
            let phase = inputs.get_mono(OscProps::PORT_ID_PHASE_CTRL);
            let mul = inputs.get_mono(OscProps::PORT_ID_MUL_CTRL);
            for (i, time) in time_range.into_iter().enumerate() {
                let freq = self.props.freq + freq.as_ref().map_or(0f32, |&freq| freq[i]);
                let phase = self.props.phase + phase.as_ref().map_or(0f32, |&phase| phase[i]);
                let mul = self.props.mul + mul.as_ref().map_or(0f32, |&mul| mul[i]);
                let val = Self::val(time as f32, freq, phase, mul);
                for &ch in &self.chs {
                    output.get_mut(ch).unwrap()[i] += val;
                }
            }
        }
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
}

impl OscSquare {
    #[must_use]
    fn new(props: OscProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        let port_props = build_port_props(false, props.channel_mask);
        Self {
            chs,
            props,
            port_props,
        }
    }

    fn val(time: f32, ds: f32, freq: f32, phase: f32, mul: f32) -> f32 {
        let val = (time * freq)
            .mul_add(std::f32::consts::TAU, phase)
            .rem_euclid(std::f32::consts::TAU)
            / std::f32::consts::TAU;
        if val > ds { 0f32 } else { mul }
    }
}

impl NodeTrait for OscSquare {
    fn process(
        &mut self,
        time_range: ResolvedTimeRange,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let sr = time_range.sr();
        let mut output = outputs.get_signals_mut(OscProps::PORT_ID_OUTPUT).unwrap();
        let events = inputs.get_events(OscProps::PORT_ID_IN_EVENTS);
        if let Some(events) = events {
            events.process(time_range, |i, time, event| {
                if let EventData::NoteOn { note, vel } = &event.data {
                    let val = Self::val(
                        (time - event.time.to_seconds(sr)) as f32,
                        self.props.ds,
                        self.props.freq * note.mul(),
                        self.props.phase,
                        self.props.mul * vel,
                    );
                    for &ch in &self.chs {
                        output.get_mut(ch).unwrap()[i] += val;
                    }
                }
            });
        } else {
            let ds = inputs.get_mono(OscProps::PORT_ID_DUTY_CTRL);
            let freq = inputs.get_mono(OscProps::PORT_ID_FREQ_CTRL);
            let phase = inputs.get_mono(OscProps::PORT_ID_PHASE_CTRL);
            let mul = inputs.get_mono(OscProps::PORT_ID_MUL_CTRL);
            for (i, time) in time_range.into_iter().enumerate() {
                let ds = self.props.ds + ds.as_ref().map_or(0f32, |ds| ds[i]).clamp(0f32, 1f32);
                let freq = self.props.freq + freq.as_ref().map_or(0f32, |freq| freq[i]);
                let phase = self.props.phase + phase.as_ref().map_or(0f32, |phase| phase[i]);
                let mul = self.props.mul + mul.as_ref().map_or(0f32, |mul| mul[i]);
                let val = Self::val(time as f32, ds, freq, phase, mul);
                for &ch in &self.chs {
                    output.get_mut(ch).unwrap()[i] += val;
                }
            }
        }
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
}

impl OscSaw {
    #[must_use]
    fn new(props: OscProps) -> Self {
        let chs = Vec::<ChannelPosition>::from(props.channel_mask);
        let port_props = build_port_props(false, props.channel_mask);
        Self {
            chs,
            props,
            port_props,
        }
    }

    fn val(time: f32, ds: f32, freq: f32, phase: f32, mul: f32) -> f32 {
        let mut val = (time * freq)
            .mul_add(std::f32::consts::TAU, phase)
            .rem_euclid(std::f32::consts::TAU)
            / std::f32::consts::TAU;
        val = if ds <= 0f32 {
            1f32 - val
        } else if ds >= 1f32 {
            val
        } else {
            if val < ds {
                val /= ds;
            } else {
                val = 1f32 - ((val - ds) / (1f32 - ds));
            }
            val
        };
        val * mul
    }
}

impl NodeTrait for OscSaw {
    fn process(
        &mut self,
        time_range: ResolvedTimeRange,
        inputs: &NodeInputs,
        outputs: &mut NodeOutputs,
    ) {
        let sr = time_range.sr();
        let mut output = outputs.get_signals_mut(OscProps::PORT_ID_OUTPUT).unwrap();
        let events = inputs.get_events(OscProps::PORT_ID_IN_EVENTS);
        if let Some(events) = events {
            events.process(time_range, |i, time, event| {
                if let EventData::NoteOn { note, vel } = &event.data {
                    let val = Self::val(
                        (time - event.time.to_seconds(sr)) as f32,
                        self.props.ds,
                        self.props.freq * note.mul(),
                        self.props.phase,
                        self.props.mul * vel,
                    );
                    for &ch in &self.chs {
                        output.get_mut(ch).unwrap()[i] += val;
                    }
                }
            });
        } else {
            let ds = inputs.get_mono(OscProps::PORT_ID_DUTY_CTRL);
            let freq = inputs.get_mono(OscProps::PORT_ID_FREQ_CTRL);
            let phase = inputs.get_mono(OscProps::PORT_ID_PHASE_CTRL);
            let mul = inputs.get_mono(OscProps::PORT_ID_MUL_CTRL);
            for (i, time) in time_range.into_iter().enumerate() {
                let ds = self.props.ds + ds.as_ref().map_or(0f32, |ds| ds[i]);
                let freq = self.props.freq + freq.as_ref().map_or(0f32, |freq| freq[i]);
                let phase = self.props.phase + phase.as_ref().map_or(0f32, |phase| phase[i]);
                let mul = self.props.mul + mul.as_ref().map_or(0f32, |mul| mul[i]);
                let val = Self::val(time as f32, ds, freq, phase, mul);
                for &ch in &self.chs {
                    output.get_mut(ch).unwrap()[i] += val;
                }
            }
        }
    }

    fn port_props(&self) -> &[PortProps] {
        &self.port_props
    }

    fn name(&self) -> &'static str {
        "Osc::Saw"
    }
}
