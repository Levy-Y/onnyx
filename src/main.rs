mod executor;
mod led_manager;
mod display_manager;
mod wifi_manager;
mod web_server;

use apa102_spi::SmartLedsWrite;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::text::Text;
use esp_idf_hal::{prelude::Peripherals, delay::FreeRtos, spi, delay, peripherals};
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::{Gpio0, PinDriver};
use esp_idf_hal::prelude::Hertz;
use esp_idf_svc::{log::EspLogger, sys::link_patches};
use log::info;
use esp_idf_hal::spi::{SpiBusDriver, SpiDriver};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::EspHttpServer;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration, EspWifi};
use esp_idf_sys::{esp_netif_init, esp_restart};
use kolibri_embedded_gui::button::Button;
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::style::medsize_rgb565_style;
use kolibri_embedded_gui::ui::Ui;
use st7735_lcd::Orientation;
use display_manager::DeviceState;
use led_manager::{init_led_manager, set_led_color};
use crate::display_manager::{init_display, init_ui, set_state};
use crate::wifi_manager::{await_network_start, get_ap_info, init_ap_modem};
// use executor::{execute_actions, parse_script, read_script_file};

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    info!("Starting...");

    let peripherals = Peripherals::take()?;
    // let mut led = init_led_manager(
    //     peripherals.spi2,
    //     peripherals.pins.gpio39,
    //     peripherals.pins.gpio40,
    // )?;
    //
    // loop {
    //     set_led_color(&mut led, 0, 255, 255)?;
    //     FreeRtos::delay_ms(1000);
    //
    //     set_led_color(&mut led, 255, 0, 255)?;
    //     FreeRtos::delay_ms(1000);
    // }
    let mut wifi = init_ap_modem(peripherals.modem, "hidden", true, "password", 1)?;
    await_network_start(&mut wifi)?;

    let ap_info = wifi.wifi().ap_netif();
    info!("SoftAP started, IP: {:?}", ap_info.get_ip_info()?.ip);
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

    set_state(&mut ui, &DeviceState::Idle(ap_info.ssid, ap_info.password, ap_info.ip))?;

    loop {
        // ui.clear_background().unwrap();
        FreeRtos::delay_ms(1000);
    }
}