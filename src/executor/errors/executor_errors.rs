use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum ExecutorError {
    TaskFailed(String),
}

#[derive(Debug)]
pub enum ParseError {
    ActionParseError(String),
    KeyParseError(String),
}


impl Display for ExecutorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ExecutorError::TaskFailed(msg) => write!(f, "Task failed: {}", msg)
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ParseError::ActionParseError(action) => write!(f, "Cannot parse the following action: {}", action),
            ParseError::KeyParseError(action) => write!(f, "Cannot parse the following key: {}", action)
        }
    }
}

impl Error for ExecutorError {}
impl Error for ParseError {}