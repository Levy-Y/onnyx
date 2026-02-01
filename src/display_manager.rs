use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::{ascii, MonoTextStyle};
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use esp_idf_hal::delay::{Ets, FreeRtos};
use esp_idf_hal::gpio::{
    Gpio0, Gpio1, Gpio2, Gpio3, Gpio39, Gpio4, Gpio5, Output, OutputPin, PinDriver,
};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::spi;
use esp_idf_hal::spi::{SpiBusDriver, SpiDeviceDriver, SpiDriver, SPI2};
use esp_idf_hal::units::Hertz;
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::style::medsize_rgb565_style;
use kolibri_embedded_gui::ui::Ui;
use st7735_lcd::{Orientation, ST7735};
use std::fmt::Display;
use log::{info, warn};

pub enum DeviceState<'a> {
    Idle(String, String, String),
    Running(&'a str),
    Finished,
}

pub type DisplayType<'a> = ST7735<
    SpiDeviceDriver<'a, SpiDriver<'a>>,
    PinDriver<'a, Gpio2, Output>,
    PinDriver<'a, Gpio1, Output>,
>;

pub fn init_display<'a>(
    spi: impl Peripheral<P = SPI2> + 'a,
    rst: impl Peripheral<P = Gpio1> + 'a,
    dc: impl Peripheral<P = Gpio2> + 'a,
    sdo: impl Peripheral<P = Gpio3> + 'a,
    cs: Option<impl Peripheral<P = Gpio4> + 'a>,
    sclk: impl Peripheral<P = Gpio5> + 'a,
) -> anyhow::Result<DisplayType<'a>> {
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

    Ok(display)
}

pub fn init_ui<'a, 'b>(
    display: &'b mut DisplayType<'a>,
) -> anyhow::Result<Ui<'b, DisplayType<'a>, Rgb565>>
where
    'a: 'b,
{
    let mut ui = Ui::new_fullscreen(display, medsize_rgb565_style());
    ui.clear_background().unwrap();
    Ok(ui)
}

pub fn set_state<'a, 'b>(
    ui: &mut Ui<'b, DisplayType<'a>, Rgb565>,
    state: &DeviceState,
) -> anyhow::Result<()>
where
    'a: 'b,
{
    match state {
        DeviceState::Idle(ssid, password, ip) => {
            ui.add(Label::new("READY").with_font(ascii::FONT_7X14_BOLD));
            ui.add(Label::new(format!("SSID: {}", ssid).as_str()).with_font(FONT_6X10));
            ui.add(Label::new(format!("PASSWORD: {}", password).as_str()).with_font(FONT_6X10));
            ui.add(Label::new(format!("IP: {}", ip).as_str()).with_font(FONT_6X10));
        }
        // DeviceState::Running(payload) => {
        //     // Draw generic "Hacking" visual
        //     Text::new("EXECUTING:", Point::zero(), style)
        //         .align_to(&display.bounding_box(), horizontal::Center, vertical::Top)
        //         .draw(display).unwrap();
        //
        //     // Use TextBox for wrapping long payload names
        //     let text_box_style = TextBoxStyleBuilder::new()
        //         .alignment(embedded_text::alignment::HorizontalAlignment::Center)
        //         .build();
        //
        //     TextBox::new(payload, display.bounding_box(), style).draw(&mut display).unwrap();
        // }
        _ => {}
    }

    Ok(())
}
