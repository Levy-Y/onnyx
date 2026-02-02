use std::fs::{create_dir, exists, read_dir};
use esp_idf_hal::gpio;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::sd::mmc::{SdMmcHostConfiguration, SdMmcHostDriver, SDMMC1};
use esp_idf_hal::sd::{SdCardConfiguration, SdCardDriver};
use esp_idf_svc::fs::fatfs::Fatfs;
use esp_idf_svc::io::vfs::MountedFatfs;

const PAYLOADS_DIR: &str = "/data/payloads";

pub struct StorageActor {
    fs: MountedFatfs<Fatfs<SdCardDriver<SdMmcHostDriver<'static>>>>,
}
pub struct File {
    name: String,
    size: u8,
    location: String,
}

impl StorageActor {
    pub fn new(
        mmc: impl Peripheral<P = SDMMC1> + 'static,
        cmd: impl Peripheral<P = gpio::Gpio15> + 'static,
        clk: impl Peripheral<P = gpio::Gpio14> + 'static,
        d0: impl Peripheral<P = gpio::Gpio2> + 'static,
        d1: impl Peripheral<P = gpio::Gpio4> + 'static,
        d2: impl Peripheral<P = gpio::Gpio12> + 'static,
        d3: impl Peripheral<P = gpio::Gpio13> + 'static,
    ) -> anyhow::Result<Self> {
        let sd_card_driver = SdCardDriver::new_mmc(
            SdMmcHostDriver::new_4bits(
                mmc,
                cmd,
                clk,
                d0,
                d1,
                d2,
                d3,
                None::<gpio::AnyIOPin>,
                None::<gpio::AnyIOPin>,
                &SdMmcHostConfiguration::new(),
            )?,
            &SdCardConfiguration::new(),
        )?;

        let mounted_fatfs = MountedFatfs::mount(Fatfs::new_sdcard(0, sd_card_driver)?, "/data", 4)?;

        if !exists(&PAYLOADS_DIR)? {
            create_dir(&PAYLOADS_DIR)?;
        };

        Ok(Self { fs: mounted_fatfs })
    }

    pub fn read_files<'a>() -> anyhow::Result<Vec<File>> {
        let mut files = vec![];

        if let Ok(directory_content) = read_dir(&PAYLOADS_DIR) {
            let entries: Vec<_> = directory_content.collect();
            if entries.len() > 0 {
                for file_result in entries {
                    let file = file_result?;
                    let f = File{
                        name: String::from(
                            file.file_name().to_str().unwrap()
                        ),
                        size: (file.metadata()?.len() / 1024) as u8,
                        location: String::from(
                            file.path().to_str().unwrap()
                        ),
                    };
                    files.push(f);
                }
            }
        }

        Ok(files)
    }

    pub fn get_file_content(file: String) -> anyhow::Result<String> {
        // TODO: Implement actual file reading from SD Card
        Ok(String::from("KEY GUI"))
    }
}
