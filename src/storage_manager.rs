use std::fs;
use esp_idf_hal::gpio;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::sd::mmc::{SdMmcHostConfiguration, SdMmcHostDriver, SDMMC1};
use esp_idf_hal::sd::{SdCardConfiguration, SdCardDriver};
use esp_idf_svc::fs::fatfs::Fatfs;
use esp_idf_svc::io::vfs::MountedFatfs;
use std::fs::{create_dir, exists, read_dir};
use std::os::unix::fs::MetadataExt;
use std::sync::mpsc::Sender;
use std::sync::OnceLock;
use anyhow::anyhow;
use log::info;
use serde::Serialize;

const PAYLOADS_DIR: &str = "/data/payloads";
pub static STORAGE: OnceLock<StorageManager> = OnceLock::new();

pub struct StorageManager {
    fs: MountedFatfs<Fatfs<SdCardDriver<SdMmcHostDriver<'static>>>>,
}
unsafe impl Send for StorageManager {}
unsafe impl Sync for StorageManager {}

#[derive(Serialize)]
pub struct File {
    name: String,
    size: u8,
    #[serde(skip_serializing)]
    location: String,
}

impl StorageManager {
    pub fn init(
        mmc: impl Peripheral<P = SDMMC1> + 'static,
        cmd: impl Peripheral<P = gpio::Gpio16> + 'static,
        clk: impl Peripheral<P = gpio::Gpio12> + 'static,
        d0: impl Peripheral<P = gpio::Gpio14> + 'static,
        d1: impl Peripheral<P = gpio::Gpio17> + 'static,
        d2: impl Peripheral<P = gpio::Gpio21> + 'static,
        d3: impl Peripheral<P = gpio::Gpio18> + 'static,
        tx: Sender<Vec<File>>
    ) -> anyhow::Result<()> {
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

        STORAGE.set(Self { fs: mounted_fatfs })
            .map_err(|_| anyhow!("StorageManager already initialized")).expect("Fatal error happened during storage initialization");

        tx.send(Self::read_files()?)?;

        Ok(())
    }

    pub fn get(&self) -> anyhow::Result<&'static Self> {
        STORAGE.get().ok_or_else(|| anyhow!("StorageManager not initialized"))
    }

    pub fn read_files<'a>() -> anyhow::Result<Vec<File>> {
        let mut files = vec![];

        if let Ok(directory_content) = read_dir(&PAYLOADS_DIR) {
            let entries: Vec<_> = directory_content.collect();
            if entries.len() > 0 {
                for file_result in entries {
                    let file = file_result?;

                    let bytes = file.metadata()?.size();
                    let size_kb = if bytes > 0 && bytes < 1024 {
                        1
                    } else {
                        (bytes / 1024) as u8
                    };

                    let f = File {
                        name: String::from(file.file_name().to_str().unwrap()),
                        size: size_kb,
                        location: String::from(file.path().to_str().unwrap()),
                    };
                    info!("Reading file with size: {} KB", &f.size);
                    files.push(f);
                }
            }
        }

        Ok(files)
    }

    pub fn get_file_content(&self, file_name: String) -> anyhow::Result<String> {
        let path = std::path::Path::new(PAYLOADS_DIR).join(file_name);
        Ok(fs::read_to_string(path)?)
    }
}
