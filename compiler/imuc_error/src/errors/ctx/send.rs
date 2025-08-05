use crate::*;

/// A marker errors used for jumping out of nested conversion calls
#[derive(Error, Debug, Clone, Copy, Default)]
#[error("sending the conversion error up")]
pub struct SendError {
    _phantom: (),
}

impl SendError {
    /// Creates a new marker error
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new marker error wrapped in type `Error`
    pub fn new_error() -> Error {
        Self::new().into()
    }
}
