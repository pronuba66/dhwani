use std::ops::Range;

use crate::channel::{ChannelPosition, ChannelPositionsMask};

#[derive(Clone)]
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

    pub(crate) fn clear(&mut self) {
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
    pub(crate) fn new(range: Range<usize>, frame: &'a mut SingalsFrame) -> Self {
        Self { range, frame }
    }

    pub fn n_channels(&self) -> u16 {
        self.frame.signals.len() as u16
    }

    pub fn clear(&mut self) {
        for (_, buffer) in self.frame.signals.iter_mut() {
            buffer[self.range.clone()].fill(0f32);
        }
    }

    pub fn get(&self, ch: ChannelPosition) -> Option<&[f32]> {
        self.frame
            .signals
            .iter()
            .find(|(c, _)| *c == ch)
            .map(|(_, buffer)| &buffer[self.range.clone()])
    }

    pub fn get_mut(&mut self, ch: ChannelPosition) -> Option<&mut [f32]> {
        let is_none = { self.frame.signals.iter().find(|(c, _)| *c == ch).is_none() };
        if is_none {
            println!("HERE, {}", self.frame.signals.len());
            for (ch, _) in self.frame.signals.iter() {
                println!("{:?}", ch);
            }
            println!("End");
        }
        self.frame
            .signals
            .iter_mut()
            .find(|(c, _)| *c == ch)
            .map(|(_, buffer)| &mut buffer[self.range.clone()])
    }
}

pub struct Signals<'a> {
    range: Range<usize>,
    frame: &'a SingalsFrame,
}

impl<'a> Signals<'a> {
    pub(crate) fn new(range: Range<usize>, frame: &'a SingalsFrame) -> Self {
        Self { range, frame }
    }

    pub fn n_channels(&self) -> u16 {
        self.frame.signals.len() as u16
    }

    pub fn get(&self, ch: ChannelPosition) -> Option<&[f32]> {
        self.frame
            .signals
            .iter()
            .find(|(c, _)| *c == ch)
            .map(|(_, buffer)| &buffer[self.range.clone()])
    }
}
