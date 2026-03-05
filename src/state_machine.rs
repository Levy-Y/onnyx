mod errors;

use crate::state_machine::errors::StateMachineError;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::LazyLock;
use strum_macros::{Display, EnumString, IntoStaticStr};

#[derive(Debug, Clone, Display, EnumString, IntoStaticStr)]
pub enum State {
    NONE,
    STARTING,
    IDLE(String, String, String),
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
        (
            State::NONE,
            vec![State::STARTING]
        ),
        (
            State::STARTING,
            vec![
                State::IDLE(String::new(), String::new(), String::new()),
                State::ERROR(String::new())
            ],
        ),
        (
            State::IDLE(String::new(), String::new(), String::new()),
            vec![State::EXECUTING(String::new()), State::ERROR(String::new())],
        ),
        (
            State::EXECUTING(String::new()),
            vec![
                State::IDLE(String::new(), String::new(), String::new()),
                State::ERROR(String::new()),
            ],
        ),
        (
            State::ERROR(String::new()),
            vec![State::IDLE(String::new(), String::new(), String::new())],
        ),
    ])
});

pub trait StateObserver: Send + Sync {
    fn on_state_change(&mut self, state: &State);
}

pub struct StateMachine {
    pub state: State,
    observers: Vec<Box<dyn StateObserver>>,
}

impl StateMachine {
    pub(crate) fn new() -> Self {
        Self {
            state: State::NONE,
            observers: Vec::new(),
        }
    }

    pub(crate) fn subscribe(&mut self, observer: Box<dyn StateObserver>) {
        self.observers.push(observer);
    }

    pub(crate) fn set(&mut self, desired: &State) -> Result<(), StateMachineError> {
        let allowed = STATE_ORDERS.get(&self.state).ok_or_else(|| {
            StateMachineError::IllegalStateSwitchError(self.state.to_string(), desired.to_string())
        })?;

        if allowed.contains(desired) {
            self.state = desired.clone();
            for observer in self.observers.iter_mut() {
                observer.on_state_change(&self.state);
            }
            Ok(())
        } else {
            Err(StateMachineError::IllegalStateSwitchError(
                self.state.to_string(),
                desired.to_string(),
            ))
        }
    }
}
