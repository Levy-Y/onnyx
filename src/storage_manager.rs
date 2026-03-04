use std::io::Write;
use anyhow::{anyhow, Error};
use esp_idf_hal::gpio;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::sd::mmc::{SdMmcHostConfiguration, SdMmcHostDriver, SDMMC1};
use esp_idf_hal::sd::{SdCardConfiguration, SdCardDriver};
use esp_idf_svc::fs::fatfs::Fatfs;
use esp_idf_svc::io::vfs::MountedFatfs;
use log::info;
use serde::Serialize;
use std::fs;
use std::fs::{create_dir, exists, read_dir, OpenOptions};
use std::os::unix::fs::MetadataExt;
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

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

const LOG_FILE: &str = "/data/debug.log";

impl StorageManager {
    pub fn init(
        mmc: impl Peripheral<P = SDMMC1> + 'static,
        cmd: impl Peripheral<P = gpio::Gpio16> + 'static,
        clk: impl Peripheral<P = gpio::Gpio12> + 'static,
        d0: impl Peripheral<P = gpio::Gpio14> + 'static,
        d1: impl Peripheral<P = gpio::Gpio17> + 'static,
        d2: impl Peripheral<P = gpio::Gpio21> + 'static,
        d3: impl Peripheral<P = gpio::Gpio18> + 'static,
        tx: Sender<Vec<File>>,
    ) -> anyhow::Result<(), Error> {
        let driver_result = SdCardDriver::new_mmc(
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
        );

        match driver_result {
            Ok(sd_card_driver) => {
                if let Ok(sd_card_fs) = Fatfs::new_sdcard(0, sd_card_driver) {
                    let mounted_fatfs = MountedFatfs::mount(sd_card_fs, "/data", 4)?;
                    if !exists(&PAYLOADS_DIR)? {
                        create_dir(&PAYLOADS_DIR)?;
                    }

                    STORAGE
                        .set(Self { fs: mounted_fatfs })
                        .map_err(|_| anyhow!("StorageManager already initialized"))
                        .expect("Storage constraint violation");

                    tx.send(Self::read_files()?)?;
                } else {
                    tx.send(vec![])?;
                    return Err(anyhow!("FATFS initialization failed."));
                }
            }
            Err(_) => {
                tx.send(vec![])?;
                return Err(anyhow!("SD card initialization timeout. Hardware absent."));
            }
        }

        Ok(())
    }

    pub fn log(msg: &str) {
        println!("{}", msg)
        // if STORAGE.get().is_none() {
        //     return;
        // }
        // if let Ok(mut f) = OpenOptions::new()
        //     .create(true)
        //     .append(true)
        //     .open(LOG_FILE)
        // {
        //     let _ = writeln!(f, "{}", msg);
        // }
    }

    pub fn get(&self) -> anyhow::Result<&'static Self> {
        STORAGE
            .get()
            .ok_or_else(|| anyhow!("StorageManager not initialized"))
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
