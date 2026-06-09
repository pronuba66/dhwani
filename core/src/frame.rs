use std::ops::Range;

use crate::{
    channel::ChannelPosition,
    event::Events,
    signal::{Signals, SignalsMut, SingalsFrame},
};

#[derive(Clone)]
pub(crate) enum Frame {
    Events(Events),
    Signals(SingalsFrame),
}

impl Frame {
    #[must_use]
    pub fn get_events<'a>(&'a self) -> Option<&'a Events> {
        match self {
            Frame::Events(events) => Some(events),
            Frame::Signals(_) => None,
        }
    }

    #[must_use]
    pub fn get_events_mut<'a>(&'a mut self) -> Option<&'a mut Events> {
        match self {
            Frame::Events(events) => Some(events),
            Frame::Signals(_) => None,
        }
    }

    #[must_use]
    pub fn get_signals<'a>(&'a self, range: Range<usize>) -> Option<Signals<'a>> {
        match self {
            Frame::Signals(frame) => Some(Signals::new(range, frame)),
            Frame::Events(_) => None,
        }
    }

    #[must_use]
    pub fn get_signals_mut<'a>(&'a mut self, range: Range<usize>) -> Option<SignalsMut<'a>> {
        match self {
            Frame::Signals(frame) => Some(SignalsMut::new(range, frame)),
            Frame::Events(_) => None,
        }
    }

    pub fn get_mono<'a>(&'a self, range: Range<usize>) -> Option<&'a [f32]> {
        match self {
            Frame::Signals(frame) => frame
                .signals
                .iter()
                .find(|(ch, _)| *ch == ChannelPosition::FrontLeft)
                .map(|(_, buffer)| &buffer[range]),
            Frame::Events(_) => None,
        }
    }
}
