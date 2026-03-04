mod errors;

use std::collections::HashMap;
use std::sync::LazyLock;
use anyhow::{Result};
use strum_macros::{Display, EnumString, IntoStaticStr};
use crate::state_machine::errors::StateMachineError;

#[derive(Debug, Clone)]
#[derive(Display, EnumString, IntoStaticStr)]
pub enum State {
    STARTING,
    IDLE,
    EXECUTING(String),
    ERROR(String),
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
impl Eq for State {}

impl std::hash::Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
    }
}

static STATE_ORDERS: LazyLock<HashMap<State, Vec<State>>> = LazyLock::new(|| {
    HashMap::from([
        (State::STARTING,                 vec![State::IDLE]),
        (State::IDLE,                     vec![State::EXECUTING(String::new()), State::ERROR(String::new())]),
        (State::EXECUTING(String::new()), vec![State::IDLE, State::ERROR(String::new())]),
        (State::ERROR(String::new()),     vec![State::IDLE]),
    ])
});

pub struct StateMachine {
    state: State
}

impl StateMachine {
    pub(crate) fn new() -> Self {
        Self { state: State::STARTING }
    }

    pub(crate) fn set(&mut self, desired: &State) -> Result<(), StateMachineError> {
        if STATE_ORDERS.get(&self.state).unwrap().contains(desired) {
            self.state = desired.clone();
            Ok(())
        } else {
            Err(StateMachineError::IllegalStateSwitchError(self.state.to_string(), desired.to_string()))
        }
    }
}