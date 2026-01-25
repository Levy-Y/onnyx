mod executor;

use esp_idf_hal::{peripheral::Peripheral, prelude::Peripherals, spi};
use executor::{execute_actions, parse_script, read_script_file};
use esp_idf_svc::{
    sys::link_patches,
    log::{
        EspLogger,
    }
};

fn main() -> anyhow::Result<()> {
    link_patches(); 
    EspLogger::initialize_default();

    let path = "/home/levi/Downloads/file.ds";
    let content = read_script_file(&path);
    let actions = parse_script(&content);

    match actions {
        Ok(a) => {
            Ok(execute_actions(a))
        },
        Err(e) => {
            Err(e.into())
        }
    }
}
