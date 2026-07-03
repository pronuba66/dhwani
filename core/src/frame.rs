use std::ops::Range;

use crate::{
    channel::ChannelPosition,
    signal::{Signals, SignalsMut, SingalsFrame},
    voice::{Voices, VoicesFrame, VoicesMut},
};

/// Private module
/// Frame is the sotrage required by output ports during a session

pub enum Frame {
    Voices(VoicesFrame),
    Signals(SingalsFrame),
}

impl Frame {
    pub fn reset(&mut self) {
        match self {
            Self::Voices(frame) => frame.clear(),
            Self::Signals(frame) => frame.reset(),
        }
    }

    #[must_use]
    pub const fn get_voices(&self) -> Option<Voices<'_>> {
        match self {
            Self::Voices(frame) => Some(Voices::new(frame)),
            Self::Signals(_) => None,
        }
    }

    #[must_use]
    pub fn get_voices_mut(&mut self) -> Option<VoicesMut<'_>> {
        match self {
            Self::Voices(frame) => Some(VoicesMut::new(frame)),
            Self::Signals(_) => None,
        }
    }

    #[must_use]
    pub const fn get_signals(&self, range: Range<usize>) -> Option<Signals<'_>> {
        match self {
            Self::Signals(frame) => Some(Signals::new(range, frame)),
            Self::Voices(_) => None,
        }
    }

    #[must_use]
    pub const fn get_signals_mut(&mut self, range: Range<usize>) -> Option<SignalsMut<'_>> {
        match self {
            Self::Signals(frame) => Some(SignalsMut::new(range, frame)),
            Self::Voices(_) => None,
        }
    }

    pub fn get_mono(&self, range: Range<usize>) -> Option<&[f32]> {
        match self {
            Self::Signals(frame) => frame
                .signals
                .iter()
                .find(|(ch, _)| *ch == ChannelPosition::FrontLeft)
                .map(|(_, buffer)| &buffer[range]),
            Self::Voices(_) => None,
        }
    }
}
