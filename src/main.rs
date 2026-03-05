mod display_manager;
mod executor;
mod led_manager;
mod state_machine;
mod storage_manager;
mod web_server;
mod wifi_manager;

use std::sync::{mpsc, LazyLock, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use anyhow::anyhow;
use crate::display_manager::DisplayManager;
use crate::executor::ExecutorActor;
use crate::state_machine::{State, StateMachine};
use crate::storage_manager::{File, StorageManager};
use crate::web_server::WebActor;
use crate::wifi_manager::{await_network_start, get_ap_info, init_ap_modem, PasswordString};
use display_manager::DeviceState;
use esp_idf_hal::{delay::FreeRtos, prelude::Peripherals};
use esp_idf_svc::{log::EspLogger, sys::link_patches};
use esp_idf_sys::{tinyusb_config_t, tinyusb_driver_install};
use log::info;

pub static STATE_MACHINE: LazyLock<Mutex<StateMachine>> =
    LazyLock::new(|| Mutex::new(StateMachine::new()));

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    info!("Starting...");

    let peripherals = Peripherals::take()?;

    let display = DisplayManager::new(
        peripherals.spi2,
        peripherals.pins.gpio1,
        peripherals.pins.gpio2,
        peripherals.pins.gpio3,
        Some(peripherals.pins.gpio4),
        peripherals.pins.gpio5,
    )?;

    STATE_MACHINE.lock().unwrap().subscribe(Box::new(display));
    STATE_MACHINE
        .lock()
        .unwrap()
        .set(&State::STARTING)?;

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
        STATE_MACHINE
            .lock()
            .unwrap()
            .set(&State::ERROR("No SD Card inserted".to_string()))?;
        info!("Storage unavailable: {}", e);
        return Err(anyhow!("Cannot continue without an SD card inserted.."))
    }

    let mut wifi = init_ap_modem(peripherals.modem, "hidden", true, "password", 1)?;
    await_network_start(&mut wifi)?;

    FreeRtos::delay_ms(2000);

    let ap_info = get_ap_info(&mut wifi)?;

    STATE_MACHINE
        .lock()
        .unwrap()
        .set(&State::IDLE(ap_info.ssid, ap_info.password, ap_info.ip))?;

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
