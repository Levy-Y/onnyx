use crate::state_machine::{State, StateObserver};
use crate::wifi_manager::PasswordString;
use anyhow::Error;
use embedded_graphics::mono_font::ascii::*;
use embedded_graphics::prelude::Size;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::{Gpio0, Gpio1, Gpio2, Gpio3, Gpio4, Gpio5, Output, PinDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::spi;
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver, SPI2};
use esp_idf_hal::units::Hertz;
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::spacer::Spacer;
use kolibri_embedded_gui::style::medsize_crt_rgb565_style;
use kolibri_embedded_gui::ui::Ui;
use st7735_lcd::{Orientation, ST7735};

pub enum DeviceState {
    Idle(String, PasswordString, String),
    Running(String),
    Finished,
    Fatal(String),
}
pub type DisplayType<'a> = ST7735<
    SpiDeviceDriver<'a, SpiDriver<'a>>,
    PinDriver<'a, Gpio2, Output>,
    PinDriver<'a, Gpio1, Output>,
>;

pub struct DisplayManager<'a> {
    display: DisplayType<'a>,
}

unsafe impl Send for DisplayManager<'_> {}
unsafe impl Sync for DisplayManager<'_> {}

impl<'a> DisplayManager<'a> {
    pub(crate) fn new<'b>(
        spi: impl Peripheral<P = SPI2> + 'b + 'a,
        rst: impl Peripheral<P = Gpio1> + 'b + 'a,
        dc: impl Peripheral<P = Gpio2> + 'b + 'a,
        sdo: impl Peripheral<P = Gpio3> + 'b + 'a,
        cs: Option<impl Peripheral<P = Gpio4> + 'b + 'a>,
        sclk: impl Peripheral<P = Gpio5> + 'b + 'a,
    ) -> Result<Self, Error> {
        let driver_config = Default::default();
        let spi_config = spi::SpiConfig::new().baudrate(Hertz::from(26_000_000));
        let spi = SpiDeviceDriver::new_single(
            spi,
            sclk,
            sdo,
            Option::<Gpio0>::None,
            cs,
            &driver_config,
            &spi_config,
        )?;

        let rst = PinDriver::output(rst)?;
        let dc = PinDriver::output(dc)?;

        let rgb = false;
        let inverted = true;
        let width = 160;
        let height = 80;

        let mut display = ST7735::new(spi, dc, rst, rgb, inverted, width, height);

        let mut delay = Ets;
        display.init(&mut delay).unwrap();
        display.set_orientation(&Orientation::Landscape).unwrap();
        display.set_offset(1, 26);

        Ok(DisplayManager { display })
    }
}

impl StateObserver for DisplayManager<'_> {
    fn on_state_change(&mut self, state: &State) {
        let mut ui = Ui::new_fullscreen(&mut self.display, medsize_crt_rgb565_style());
        ui.clear_background().unwrap();

        match state {
            State::STARTING => {
                ui.add(Spacer::new(Size::new(160, 26)));
                ui.add(Label::new("INITIALIZING...").with_font(FONT_7X14_BOLD));
            }

            State::IDLE(ssid, password, ip) => {
                ui.add(Label::new("READY").with_font(FONT_7X14_BOLD));
                ui.add(Label::new(format!("SSID: {}", ssid).as_str()).with_font(FONT_6X10));
                ui.add(
                    Label::new(format!("PASSWORD: {}", password.as_str()).as_str())
                        .with_font(FONT_6X10),
                );
                ui.add(Label::new(format!("IP: {}", ip).as_str()).with_font(FONT_6X10));
            }

            State::EXECUTING(task) => {
                ui.add(Spacer::new(Size::new(160, 26)));
                ui.add(Label::new("Running script").with_font(FONT_7X14_BOLD));
                ui.add(Label::new(task).with_font(FONT_7X14_BOLD));
            }

            State::ERROR(msg) => {
                ui.add(Spacer::new(Size::new(160, 26)));
                ui.add(Label::new(msg).with_font(FONT_7X14_BOLD));
                ui.add(Label::new("Manual reset required!").with_font(FONT_7X14_BOLD));
            }
            _ => {}
        }
    }
}
