use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum StateMachineError {
    IllegalStateSwitchError(String, String),
}

impl Display for StateMachineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            StateMachineError::IllegalStateSwitchError(previous, desired) => write!(f, "Illegal state switch: {} -> {}", previous, desired),
        }
    }
}

impl Error for StateMachineError {}
