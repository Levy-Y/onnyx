use apa102_spi::{Apa102Writer, PixelOrder, SmartLedsWrite, RGB8};
use esp_idf_hal::gpio::{Gpio39, Gpio40};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::{Hertz, Peripherals};
use esp_idf_hal::spi::config::DriverConfig;
use esp_idf_hal::spi::{config, SpiBusDriver, SpiDriver, SpiError, SPI2};

pub fn init_led_manager<'a>(
        spi_periph: SPI2,
        sclk: impl Peripheral<P = Gpio39> + 'a,
        mosi: impl Peripheral<P = Gpio40> + 'a,
    ) -> anyhow::Result<Apa102Writer<SpiBusDriver<'a, SpiDriver<'a>>>> {

    let config = DriverConfig::new();
    let spi_driver = SpiDriver::new(
        spi_periph,
        sclk,
        mosi,
        None::<esp_idf_hal::gpio::AnyIOPin>,
        &config,
    )?;

    let config = config::Config::new().baudrate(Hertz::from(2_000_000));
    let spi_bus = SpiBusDriver::new(spi_driver, &config)?;

    Ok(Apa102Writer::new(spi_bus, 4, PixelOrder::default()))
}

pub fn set_led_color<'a>(led_driver: &mut Apa102Writer<SpiBusDriver<'a, SpiDriver<'a>>>, b: u8, g: u8, r: u8) -> anyhow::Result<(), SpiError> {
    led_driver.write([RGB8::new(r, g, b)])
}