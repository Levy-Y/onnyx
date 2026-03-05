use esp_idf_sys::{hid_report_type_t, tusb_desc_device_t};
use std::ffi::c_char;

pub static DEVICE_DESC: tusb_desc_device_t = tusb_desc_device_t {
    bLength: 18,
    bDescriptorType: 0x01,
    bcdUSB: 0x0200,
    bDeviceClass: 0x00,
    bDeviceSubClass: 0x00,
    bDeviceProtocol: 0x00,
    bMaxPacketSize0: 64,
    idVendor: 0x303A,
    idProduct: 0x4002,
    bcdDevice: 0x0100,
    iManufacturer: 0x01,
    iProduct: 0x02,
    iSerialNumber: 0x03,
    bNumConfigurations: 0x01,
};

pub static REPORT_DESC: [u8; 45] = [
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x06, // Usage (Keyboard)
    0xa1, 0x01, // Collection (Application)
    0x05, 0x07, // Usage Page (Key Codes)
    0x19, 0xe0, // Usage Minimum (224)
    0x29, 0xe7, // Usage Maximum (231)
    0x15, 0x00, // Logical Minimum (0)
    0x25, 0x01, // Logical Maximum (1)
    0x75, 0x01, // Report Size (1)
    0x95, 0x08, // Report Count (8)
    0x81, 0x02, // Input (Data, Variable, Absolute) - modifier
    0x95, 0x01, // Report Count (1)
    0x75, 0x08, // Report Size (8)
    0x81, 0x01, // Input (Constant) - reserved
    0x95, 0x06, // Report Count (6)
    0x75, 0x08, // Report Size (8)
    0x15, 0x00, // Logical Minimum (0)
    0x25, 0x65, // Logical Maximum (101)
    0x05, 0x07, // Usage Page (Key Codes)
    0x19, 0x00, // Usage Minimum (0)
    0x29, 0x65, // Usage Maximum (101)
    0x81, 0x00, // Input (Data, Array) - keycodes
    0xc0, // End Collection
];

pub static CONFIG_DESC: [u8; 34] = [
    // Configuration Descriptor
    0x09, 0x02, 0x22, 0x00, 0x01, 0x01, 0x00, 0xA0, 0x32, // Interface Descriptor
    0x09, 0x04, 0x00, 0x00, 0x01, 0x03, 0x00, 0x00, 0x00, // HID Descriptor
    0x09, 0x21, 0x11, 0x01, 0x00, 0x01, 0x22, 0x2D, 0x00, // Endpoint Descriptor
    0x07, 0x05, 0x81, 0x03, 0x10, 0x00, 0x0A,
];

pub static STR_LANG: [u8; 4] = [0x09, 0x04, 0x00, 0x00];
pub static STR_MFG: &[u8] = b"Manufacturer\0";
pub static STR_PROD: &[u8] = b"Device\0";
pub static STR_SER: &[u8] = b"123456\0";

pub struct SyncStringDesc(pub(crate) [*const c_char; 4]);
unsafe impl Sync for SyncStringDesc {}

pub static STRING_DESC: SyncStringDesc = SyncStringDesc([
    STR_LANG.as_ptr() as *const c_char,
    STR_MFG.as_ptr() as *const c_char,
    STR_PROD.as_ptr() as *const c_char,
    STR_SER.as_ptr() as *const c_char,
]);

#[unsafe(no_mangle)]
pub extern "C" fn tud_hid_descriptor_report_cb(_instance: u8) -> *const u8 {
    REPORT_DESC.as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn tud_hid_get_report_cb(
    _instance: u8,
    _report_id: u8,
    _report_type: hid_report_type_t,
    _buffer: *mut u8,
    _reqlen: u16,
) -> u16 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn tud_hid_set_report_cb(
    _instance: u8,
    _report_id: u8,
    _report_type: hid_report_type_t,
    _buffer: *const u8,
    _bufsize: u16,
) {
}
