use crate::*;

/// A marker errors used for jumping out of nested conversion calls
#[derive(Error, Debug, Clone, Copy, Default)]
#[error("sending the conversion error up")]
pub struct SendError {
    _phantom: (),
}

impl SendError {
    pub fn new() -> Self {
        Self::default()
    }
}
