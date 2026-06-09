use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
}

impl Error {
    #[must_use]
    pub const fn msg(msg: String) -> Self {
        Self { msg }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Dhwani error")?;
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}
