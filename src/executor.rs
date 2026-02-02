mod enums;

mod errors {
    pub mod executor_errors;
}

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
pub use crate::executor::errors::executor_errors::ExecutorError;
pub use enums::Actions;
use crate::storage_manager::get_file_content;
// use std::fs;

// pub fn read_script_file(path: &str) -> String {
//     fs::read_to_string(path).unwrap()
// }

pub struct ExecutorActor;

impl ExecutorActor {
    pub fn start(rx: Receiver<String>) -> anyhow::Result<Self> {
        'poller: loop {
            match rx.try_recv() {
                Ok(msg) => Self::handle_action(msg)?,
                Err(mpsc::TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(100));
                }
                Err(mpsc::TryRecvError::Disconnected) => break 'poller,
            }
        }

        Ok(Self)
    }

    fn handle_action(script_name: String) -> anyhow::Result<()> {
        let script_content = get_file_content(script_name)?;
        let parsed_actions = Self::parse_script(&script_content)?;
        Self::execute_actions(parsed_actions);

        Ok(())
    }
    
    fn parse_script(script_content: &String) -> Result<Vec<Actions>, ExecutorError> {
        if script_content.len() == 0 {
            return Err(ExecutorError::TaskFailed(String::from(
                "Cannot parse an empty file.",
            )));
        }
    
        let mut actions: Vec<Actions> = vec![];
    
        for line in script_content.lines() {
            let action = Actions::from_line(line);
            if action.is_err() {
                return Err(ExecutorError::TaskFailed(
                    "Error while parsing script.".to_string(),
                ));
            }
    
            actions.push(action.unwrap());
        }
    
        Ok(actions)
    }
    
    fn execute_actions(actions: Vec<Actions>) {
        for action in &actions {
            match action {
                // TODO: These are only placeholder actions, must implement the actual esp32 solution to run these commands
                Actions::WRITE(s) => println!("Typing string: \"{}\"", s),
                Actions::WAIT(t) => println!("Waiting for {}s", t),
                Actions::KEY(keys) => {
                    for key in keys {
                        println!("Pressing key: {}", key);
                    }
                }
                _ => {}
            }
        }
    }
}