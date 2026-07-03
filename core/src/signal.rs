use std::ops::Range;

use crate::{
    Error,
    channel::{ChannelPosition, ChannelPositionsMask},
};

pub(crate) struct SingalsFrame {
    pub(crate) signals: Vec<(ChannelPosition, Vec<f32>)>,
}

impl SingalsFrame {
    pub(crate) fn new(channel_mask: ChannelPositionsMask, buffer_size: usize) -> Self {
        let n_channels = channel_mask.bits().count_ones() as usize;
        let mut signals = Vec::<(ChannelPosition, Vec<f32>)>::with_capacity(n_channels);
        for ch in channel_mask.iter() {
            let buffer = vec![0f32; buffer_size];
            signals.push((ch.try_into().unwrap(), buffer));
        }
        Self { signals }
    }

    pub(crate) fn reset(&mut self) {
        for (_, buffer) in &mut self.signals {
            buffer.fill(0f32);
        }
    }
}

pub struct SignalsMut<'a> {
    range: Range<usize>,
    frame: &'a mut SingalsFrame,
}

impl<'a> SignalsMut<'a> {
    pub(crate) const fn new(range: Range<usize>, frame: &'a mut SingalsFrame) -> Self {
        Self { range, frame }
    }

    #[must_use]
    pub const fn n_channels(&self) -> u16 {
        self.frame.signals.len() as u16
    }

    pub const fn len(&self) -> usize {
        self.range.end - self.range.start
    }

    pub fn clear(&mut self) {
        for (_, buffer) in &mut self.frame.signals {
            buffer[self.range.clone()].fill(0f32);
        }
    }

    #[must_use]
    pub fn get(&self, ch: ChannelPosition) -> Option<&[f32]> {
        self.frame
            .signals
            .iter()
            .find(|(c, _)| *c == ch)
            .map(|(_, buffer)| &buffer[self.range.clone()])
    }

    #[must_use]
    pub fn get_mut(&mut self, ch: ChannelPosition) -> Option<&mut [f32]> {
        self.frame
            .signals
            .iter_mut()
            .find(|(c, _)| *c == ch)
            .map(|(_, buffer)| &mut buffer[self.range.clone()])
    }

    pub fn copy(&mut self, dst_ch: ChannelPosition, src_ch: ChannelPosition) -> Result<(), Error> {
        let dst = self
            .frame
            .signals
            .iter()
            .position(|(ch, _)| *ch == dst_ch)
            .ok_or_else(|| Error::msg("Destination not found".into()))?;
        let src = self
            .frame
            .signals
            .iter()
            .position(|(ch, _)| *ch == src_ch)
            .ok_or_else(|| Error::msg("Source not found".into()))?;
        let signals = &mut self.frame.signals;
        if dst < src {
            let (left, right) = signals.split_at_mut(src);
            left[dst].1.copy_from_slice(&right[0].1);
        } else {
            let (left, right) = signals.split_at_mut(dst);
            right[0].1.copy_from_slice(&left[src].1);
        }
        Ok(())
    }
}

pub struct Signals<'a> {
    range: Range<usize>,
    frame: &'a SingalsFrame,
}

impl<'a> Signals<'a> {
    pub(crate) const fn new(range: Range<usize>, frame: &'a SingalsFrame) -> Self {
        Self { range, frame }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn n_channels(&self) -> u16 {
        self.frame.signals.len() as u16
    }

    #[must_use]
    pub fn get(&self, ch: ChannelPosition) -> Option<&[f32]> {
        self.frame
            .signals
            .iter()
            .find(|(c, _)| *c == ch)
            .map(|(_, buffer)| &buffer[self.range.clone()])
    }
}
