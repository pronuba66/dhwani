use std::ops::Range;

use crate::{
    channel::ChannelPosition,
    event::{Events, EventsFrame, EventsMut},
    signal::{Signals, SignalsMut, SingalsFrame},
    time::SampleRateBaseType,
};

/// Private module
/// Frame is the sotrage required by output ports during a session

pub enum Frame {
    Events(EventsFrame),
    Signals(SingalsFrame),
}

impl Frame {
    pub fn reset(&mut self) {
        match self {
            Self::Events(frame) => frame.clear(),
            Self::Signals(frame) => frame.reset(),
        }
    }

    #[must_use]
    pub const fn get_events(&self, sr: SampleRateBaseType) -> Option<Events<'_>> {
        match self {
            Self::Events(frame) => Some(Events::new(sr, frame)),
            Self::Signals(_) => None,
        }
    }

    #[must_use]
    pub fn get_events_mut(&mut self, sr: SampleRateBaseType) -> Option<EventsMut<'_>> {
        match self {
            Self::Events(frame) => Some(EventsMut::new(sr, frame)),
            Self::Signals(_) => None,
        }
    }

    #[must_use]
    pub const fn get_signals(&self, range: Range<usize>) -> Option<Signals<'_>> {
        match self {
            Self::Signals(frame) => Some(Signals::new(range, frame)),
            Self::Events(_) => None,
        }
    }

    #[must_use]
    pub const fn get_signals_mut(&mut self, range: Range<usize>) -> Option<SignalsMut<'_>> {
        match self {
            Self::Signals(frame) => Some(SignalsMut::new(range, frame)),
            Self::Events(_) => None,
        }
    }

    pub fn get_mono(&self, range: Range<usize>) -> Option<&[f32]> {
        match self {
            Self::Signals(frame) => frame
                .signals
                .iter()
                .find(|(ch, _)| *ch == ChannelPosition::FrontLeft)
                .map(|(_, buffer)| &buffer[range]),
            Self::Events(_) => None,
        }
    }
}
