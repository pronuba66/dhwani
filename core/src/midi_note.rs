use std::fmt::Debug;

use crate::{
    Error,
    midi_consts::{MIDI_NOTE_MULS, MIDI_NOTE_NAMES},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MidiNote {
    num: usize,
}

impl MidiNote {
    ///
    /// # Errors
    /// Return `Err` if `note` is an invalid midi note
    ///
    pub fn from_midi_str(note: &str) -> Result<Self, Error> {
        if note.len() < 2 {
            return Err(Error::msg("Invalid note".into()));
        }
        let bytes = note.as_bytes();
        let base = bytes[0].to_ascii_uppercase();
        let mut pitch = match base {
            b'C' => 0,
            b'D' => 2,
            b'E' => 4,
            b'F' => 5,
            b'G' => 7,
            b'A' => 9,
            b'B' => 11,
            _ => return Err(Error::msg("Invalid note".into())),
        };
        let mut idx = 1;

        if note.len() > 2 {
            match bytes[1] {
                b'#' => {
                    // only allow real sharps
                    match base {
                        b'C' | b'D' | b'F' | b'G' | b'A' => pitch += 1,
                        _ => return Err(Error::msg("Invalid sharp note".into())),
                    }
                    idx = 2;
                }
                b'b' => {
                    // only allow real flats
                    match base {
                        b'D' | b'E' | b'G' | b'A' | b'B' => pitch -= 1,
                        _ => return Err(Error::msg("Invalid flat note".into())),
                    }
                    idx = 2;
                }
                _ => {}
            }
        }
        let octave: i32 = note[idx..]
            .parse()
            .map_err(|_| Error::msg("Invalid octave".into()))?;

        let midi_num = (octave + 1) * 12 + pitch;
        let midi_num = u8::try_from(midi_num).map_err(|_| Error::msg("Invalid octave".into()))?;
        Self::from_midi_num(midi_num)
    }

    ///
    /// # Errors
    /// Return `Err` if `num` is an invalid midi num
    ///
    pub fn from_midi_num(num: u8) -> Result<Self, Error> {
        let num = num as usize;
        if num < MIDI_NOTE_MULS.len() {
            Ok(Self { num })
        } else {
            Err(Error::msg("Invalid midi note".into()))
        }
    }

    #[must_use]
    pub const fn mul(&self) -> f32 {
        MIDI_NOTE_MULS[self.num]
    }

    /// Angular frequency in rad/s
    #[must_use]
    pub fn w(&self) -> f32 {
        440f32 * std::f32::consts::TAU * MIDI_NOTE_MULS[self.num]
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        MIDI_NOTE_NAMES[self.num % 12]
    }

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn octave(&self) -> i8 {
        (self.num / 12) as i8 - 1
    }
}

impl Debug for MidiNote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.name(), self.octave())
    }
}
