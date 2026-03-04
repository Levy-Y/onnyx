mod display_manager;
mod executor;
mod led_manager;
mod storage_manager;
mod web_server;
mod wifi_manager;
mod state_machine;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::display_manager::{init_display, init_ui, set_state};
use crate::executor::ExecutorActor;
use crate::storage_manager::{File, StorageManager};
use crate::web_server::WebActor;
use crate::wifi_manager::{await_network_start, get_ap_info, init_ap_modem, PasswordString};
use display_manager::DeviceState;
use esp_idf_hal::{delay::FreeRtos, prelude::Peripherals};
use esp_idf_svc::{log::EspLogger, sys::link_patches};
use esp_idf_sys::{tinyusb_config_t, tinyusb_driver_install};
use log::info;
use crate::state_machine::{State, StateMachine};

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    let mut state_machine = StateMachine::new();
    state_machine.set(&State::STARTING)?;

    info!("Starting...");

    let peripherals = Peripherals::take()?;
    let mut wifi = init_ap_modem(peripherals.modem, "hidden", true, "password", 1)?;
    await_network_start(&mut wifi)?;

    let mut display = init_display(
        peripherals.spi2,
        peripherals.pins.gpio1,
        peripherals.pins.gpio2,
        peripherals.pins.gpio3,
        Some(peripherals.pins.gpio4),
        peripherals.pins.gpio5,
    )?;

    let mut ui = init_ui(&mut display)?;
    let ap_info = get_ap_info(&mut wifi)?;

    state_machine.set(&State::IDLE)?;
    set_state(
        &mut ui,
        &DeviceState::Idle(
            ap_info.ssid,
            PasswordString::new(ap_info.password)?,
            ap_info.ip,
        ),
    )?;

    let (web_executor_tx, web_executor_rx) = mpsc::channel::<String>();
    let (storage_web_tx, storage_web_rx) = mpsc::channel::<Vec<File>>();

    let tx_clone1 = web_executor_tx.clone();

    if let Err(e) = StorageManager::init(
        peripherals.sdmmc1,
        peripherals.pins.gpio16,
        peripherals.pins.gpio12,
        peripherals.pins.gpio14,
        peripherals.pins.gpio17,
        peripherals.pins.gpio21,
        peripherals.pins.gpio18,
        storage_web_tx,
    ) {
        state_machine.set(&State::ERROR("No SD Card inserted".to_string()))?;
        info!("Storage unavailable: {}", e);
    }

    thread::Builder::new()
        .stack_size(8192)
        .name("executor".into())
        .spawn(move || ExecutorActor::start(web_executor_rx))?;

    thread::Builder::new()
        .stack_size(8192)
        .name("web_actor".into())
        .spawn(move || {
            let _web_actor = WebActor::start(80, tx_clone1, storage_web_rx).unwrap();

            loop {
                thread::sleep(Duration::from_millis(1000));
            }
        })?;
    loop {
        FreeRtos::delay_ms(1000);
    }
}
