mod enums;
mod tinyusb;
pub mod errors;
use std::thread;

pub use errors::ExecutorError;
use crate::executor::tinyusb::{CONFIG_DESC, DEVICE_DESC, STRING_DESC};
use crate::storage_manager::STORAGE;
use crate::StorageManager;
pub use enums::Actions;
use esp_idf_sys::{
    tinyusb_config_t, tinyusb_driver_install, tud_hid_n_keyboard_report, tud_hid_n_ready,
    tud_mounted,
};
use std::ffi::c_char;
use std::sync::mpsc::Receiver;
use std::sync::OnceLock;
use std::time::Duration;

static HID_INITIALIZED: OnceLock<()> = OnceLock::new();

pub struct ExecutorActor;

impl ExecutorActor {
    pub fn start(rx: Receiver<String>) -> anyhow::Result<Self> {
        StorageManager::log("Executor actor started");
        loop {
            match rx.recv() {
                Ok(msg) => {
                    StorageManager::log(&format!("Executor handling action: {}", msg));
                    if let Err(e) = Self::handle_action(msg) {
                        StorageManager::log(&format!("Executor handling action: {}", e));
                    }
                }
                Err(_) => {
                    StorageManager::log("Executor channel closed, shutting down.");
                    break;
                }
            }
        }

        Ok(Self)
    }

    fn ensure_hid_initialized() {
        HID_INITIALIZED.get_or_init(|| {
            StorageManager::log("Initializing TinyUSB HID...");

            unsafe {
                let mut cfg: tinyusb_config_t = Default::default();

                cfg.string_descriptor = STRING_DESC.0.as_ptr() as *mut *const c_char;
                cfg.string_descriptor_count = STRING_DESC.0.len() as i32;

                cfg.__bindgen_anon_1 = esp_idf_sys::tinyusb_config_t__bindgen_ty_1 {
                    device_descriptor: DEVICE_DESC.as_ptr()
                        as *const esp_idf_sys::tusb_desc_device_t,
                };
                cfg.__bindgen_anon_2
                    .__bindgen_anon_1
                    .configuration_descriptor = CONFIG_DESC.as_ptr();

                let err = tinyusb_driver_install(&cfg);
                if err != 0 {
                    StorageManager::log(&format!("TinyUSB driver install failed: {}", err));
                } else {
                    StorageManager::log("TinyUSB HID initialized.");
                }
            }
        });
    }

    fn handle_action(script_name: String) -> anyhow::Result<()> {
        if script_name.starts_with("DIRECT:KEY ") {
            let key_str = &script_name[11..];
            let action = Actions::from_line(&format!("KEY {}", key_str))?;
            Self::execute_actions(vec![action]);
            return Ok(());
        }
        let script_content = STORAGE.get().unwrap().get_file_content(script_name)?;
        let parsed_actions = Self::parse_script(&script_content)?;
        Self::execute_actions(parsed_actions);

        Ok(())
    }

    fn parse_script(script_content: &String) -> Result<Vec<Actions>, ExecutorError> {
        if script_content.is_empty() {
            return Err(ExecutorError::TaskFailed(String::from(
                "Cannot parse an empty file.",
            )));
        }

        let mut actions: Vec<Actions> = vec![];

        for line in script_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            let action = Actions::from_line(line);
            if action.is_err() {
                return Err(ExecutorError::TaskFailed(
                    "Error while parsing script.".to_string(),
                ));
            }

            actions.push(action.unwrap());
        }

        Ok(actions)
    }

    fn execute_actions(actions: Vec<Actions>) {
        Self::ensure_hid_initialized();

        for action in &actions {
            match action {
                Actions::WRITE(s) => {
                    for c in s.chars() {
                        let (modifier, keycode) = Self::char_to_keycode(c);
                        if keycode != 0 {
                            Self::send_report(modifier, [keycode, 0, 0, 0, 0, 0]);
                        }
                    }
                }
                Actions::WAIT(t) => {
                    thread::sleep(Duration::from_millis((*t as u64) * 1000));
                }
                Actions::KEY(keys) => {
                    let mut keycodes = [0u8; 6];
                    let mut modifier = 0u8;
                    let mut count = 0;

                    for key in keys {
                        match key {
                            enums::Keys::GUI => modifier |= 0x08,
                            enums::Keys::ALT => modifier |= 0x04,
                            enums::Keys::CTRL => modifier |= 0x01,
                            enums::Keys::SHIFT => modifier |= 0x02,
                            enums::Keys::CHAR(c) => {
                                let (m, k) = Self::char_to_keycode(*c);
                                modifier |= m;
                                if k != 0 && count < 6 {
                                    keycodes[count] = k;
                                    count += 1;
                                }
                            }
                            enums::Keys::ENTER => {
                                if count < 6 {
                                    keycodes[count] = 0x28;
                                    count += 1;
                                }
                            }
                            enums::Keys::CAPS => {
                                if count < 6 {
                                    keycodes[count] = 0x39;
                                    count += 1;
                                }
                            }
                            enums::Keys::DEL => {
                                if count < 6 {
                                    keycodes[count] = 0x4c;
                                    count += 1;
                                }
                            }
                            enums::Keys::ESC => {
                                if count < 6 {
                                    keycodes[count] = 0x29;
                                    count += 1;
                                }
                            }
                            enums::Keys::TAB => {
                                if count < 6 {
                                    keycodes[count] = 0x2b;
                                    count += 1;
                                }
                            }
                            enums::Keys::PRTSCR => {
                                if count < 6 {
                                    keycodes[count] = 0x46;
                                    count += 1;
                                }
                            }
                            enums::Keys::HOME => {
                                if count < 6 {
                                    keycodes[count] = 0x4a;
                                    count += 1;
                                }
                            }
                            enums::Keys::END => {
                                if count < 6 {
                                    keycodes[count] = 0x4d;
                                    count += 1;
                                }
                            }
                            enums::Keys::PGUP => {
                                if count < 6 {
                                    keycodes[count] = 0x4b;
                                    count += 1;
                                }
                            }
                            enums::Keys::PGDN => {
                                if count < 6 {
                                    keycodes[count] = 0x4e;
                                    count += 1;
                                }
                            }
                        }
                    }
                    Self::send_report(modifier, keycodes);
                }
                _ => {}
            }
        }
    }

    fn send_report(modifier: u8, mut keycodes: [u8; 6]) {
        unsafe {
            while !tud_mounted() {
                thread::sleep(Duration::from_millis(10));
            }
            while !tud_hid_n_ready(0) {
                thread::sleep(Duration::from_millis(10));
            }

            tud_hid_n_keyboard_report(0, 0, modifier, keycodes.as_mut_ptr());
            thread::sleep(Duration::from_millis(10));

            let mut release = [0u8; 6];
            tud_hid_n_keyboard_report(0, 0, 0, release.as_mut_ptr());
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn char_to_keycode(c: char) -> (u8, u8) {
        let mut modifier = 0;
        let keycode = match c {
            'a'..='z' => 0x04 + (c as u8 - b'a'),
            'A'..='Z' => {
                modifier = 0x02;
                0x04 + (c as u8 - b'A')
            }
            '1'..='9' => 0x1e + (c as u8 - b'1'),
            '0' => 0x27,
            ' ' => 0x2c,
            '\n' => 0x28,
            '!' => {
                modifier = 0x02;
                0x1e
            }
            '@' => {
                modifier = 0x02;
                0x1f
            }
            '#' => {
                modifier = 0x02;
                0x20
            }
            '$' => {
                modifier = 0x02;
                0x21
            }
            '%' => {
                modifier = 0x02;
                0x22
            }
            '^' => {
                modifier = 0x02;
                0x23
            }
            '&' => {
                modifier = 0x02;
                0x24
            }
            '*' => {
                modifier = 0x02;
                0x25
            }
            '(' => {
                modifier = 0x02;
                0x26
            }
            ')' => {
                modifier = 0x02;
                0x27
            }
            '-' => 0x2d,
            '_' => {
                modifier = 0x02;
                0x2d
            }
            '=' => 0x2e,
            '+' => {
                modifier = 0x02;
                0x2e
            }
            '[' => 0x2f,
            '{' => {
                modifier = 0x02;
                0x2f
            }
            ']' => 0x30,
            '}' => {
                modifier = 0x02;
                0x30
            }
            '\\' => 0x31,
            '|' => {
                modifier = 0x02;
                0x31
            }
            ';' => 0x33,
            ':' => {
                modifier = 0x02;
                0x33
            }
            '\'' => 0x34,
            '"' => {
                modifier = 0x02;
                0x34
            }
            '`' => 0x35,
            '~' => {
                modifier = 0x02;
                0x35
            }
            ',' => 0x36,
            '<' => {
                modifier = 0x02;
                0x36
            }
            '.' => 0x37,
            '>' => {
                modifier = 0x02;
                0x37
            }
            '/' => 0x38,
            '?' => {
                modifier = 0x02;
                0x38
            }
            _ => 0,
        };
        (modifier, keycode)
    }
}
