use std::fmt::Debug;

use crate::{Error, port::Port};

#[derive(Clone, Copy)]
pub struct Connection {
    pub(crate) source: Port,
    pub(crate) target: Port,
}

impl Connection {
    pub(crate) fn new(source: Port, target: Port) -> Result<Self, Error> {
        if !source.kind.is_output() {
            return Err(Error::msg("Source port must be output".into()));
        }
        if !target.kind.is_input() {
            return Err(Error::msg("Target port must be input".into()));
        }
        if source.kind.is_voice() != target.kind.is_voice() {
            return Err(Error::msg("Mismatch port type".into()));
        }
        Ok(Self { source, target })
    }
}

impl Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} => {:?}", self.source, self.target)
    }
}
