mod enums;

mod errors {
    pub mod executor_errors;
}

pub use crate::executor::errors::executor_errors::ExecutorError;
pub use enums::Actions;
use std::fs;

pub fn read_script_file(path: &str) -> String {
    fs::read_to_string(path).unwrap()
}

pub fn parse_script(script_content: &String) -> Result<Vec<Actions>, ExecutorError> {
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

pub fn execute_actions(actions: Vec<Actions>) {
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
