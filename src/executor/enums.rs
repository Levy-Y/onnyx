use crate::executor::enums::Keys::{
    ALT, CAPS, CTRL, DEL, END, ENTER, ESC, GUI, HOME, PGDN, PGUP, PRTSCR, SHIFT, TAB,
};
use crate::executor::errors::executor_errors::ParseError;
use crate::executor::errors::executor_errors::ParseError::{ActionParseError, KeyParseError};
use crate::executor::Actions::{COMMENT, KEY, WAIT, WRITE};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub enum Actions {
    WRITE(String),
    WAIT(u16),
    KEY(Vec<Keys>),
    COMMENT,
}

#[derive(Debug)]
pub enum Keys {
    GUI,
    ALT,
    CTRL,
    SHIFT,
    CAPS,
    DEL,
    ESC,
    TAB,
    PRTSCR,
    HOME,
    END,
    PGUP,
    PGDN,
    ENTER,
}

impl Actions {
    pub fn from_line(line: &str) -> Result<Self, ParseError> {
        let line = line.trim();
        let (command, arg) = line.split_once(' ').unwrap_or((line, ""));

        match command {
            "WRITE" => {
                let trimmed = &arg.trim();
                if trimmed.starts_with('"') && trimmed.ends_with('"') {
                    let unquoted = &trimmed[1..trimmed.len() - 1];
                    Ok(WRITE(String::from(unquoted)))
                } else {
                    Ok(WRITE(String::from(arg)))
                }
            }
            "WAIT" => Ok(WAIT(arg.parse::<u16>().unwrap_or(0))),
            "KEY" => {
                let mut keys: Vec<Keys> = vec![];
                if !arg.contains('+') {
                    let key = Keys::from_str(arg);
                    if key.is_err() {
                        return Err(KeyParseError(
                            "Cannot parse action argument, exiting...".to_string(),
                        ));
                    }
                    keys.push(key?);
                    return Ok(KEY(keys));
                }

                let split_arg: Vec<&str> = arg.split('+').collect();
                for key in split_arg {
                    let key = Keys::from_str(key);
                    if key.is_err() {
                        return Err(KeyParseError(
                            "Cannot parse action argument, exiting...".to_string(),
                        ));
                    }
                    keys.push(key?);
                }
                Ok(KEY(keys))
            }
            "//" => Ok(COMMENT),
            _ => Err(ActionParseError(String::from(command))),
        }
    }
}

impl Display for Keys {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for Keys {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GUI" => Ok(GUI),
            "ALT" => Ok(ALT),
            "CTRL" => Ok(CTRL),
            "SHIFT" => Ok(SHIFT),
            "CAPS" => Ok(CAPS),
            "DEL" => Ok(DEL),
            "ESC" => Ok(ESC),
            "TAB" => Ok(TAB),
            "PRTSCR" => Ok(PRTSCR),
            "HOME" => Ok(HOME),
            "END" => Ok(END),
            "PGUP" => Ok(PGUP),
            "PGDN" => Ok(PGDN),
            "ENTER" => Ok(ENTER),
            _ => Err(KeyParseError(s.to_string())),
        }
    }
}
