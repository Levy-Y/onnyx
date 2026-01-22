mod executor;

use executor::{read_script_file, parse_script, execute_actions};

fn main() {
    let path = "/home/levi/Downloads/file.ds";
    let content = read_script_file(&path);
    let actions = parse_script(&content);
    
    if actions.is_err() {
        println!("Error while executing: {}", actions.err().unwrap().to_string())
    } else {
        execute_actions(actions.unwrap())
    }
}