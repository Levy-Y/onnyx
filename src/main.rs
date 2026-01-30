mod executor;
mod led_manager;

use apa102_spi::SmartLedsWrite;
use esp_idf_hal::{prelude::Peripherals, delay::FreeRtos};
use esp_idf_svc::{log::EspLogger, sys::link_patches};
use log::info;
use esp_idf_hal::spi::{SpiBusDriver, SpiDriver};
use led_manager::{init_led_manager, set_led_color};
// use executor::{execute_actions, parse_script, read_script_file};

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    info!("Starting...");

    let peripherals = Peripherals::take()?;
    let mut led = init_led_manager(
        peripherals.spi2,
        peripherals.pins.gpio39,
        peripherals.pins.gpio40,
    )?;

    loop {
        set_led_color(&mut led, 0, 255, 255)?;
        FreeRtos::delay_ms(1000);

        set_led_color(&mut led, 255, 0, 255)?;
        FreeRtos::delay_ms(1000);
    }

    // let path = "/home/levi/Downloads/file.ds";
    // let content = read_script_file(&path);
    // let actions = parse_script(&content);
    //
    // match actions {
    //     Ok(a) => Ok(execute_actions(a)),
    //     Err(e) => Err(e.into()),
    // }
    Ok(())
}