//! The device's authentication key (NVS), its id (base MAC) and hardware random
//! numbers (audit Phase 2B, PIN-TPL-003).
//!
//! A device has no key until the desktop asks for one over the USB console
//! (`ProvisionKey`); until then it is unprovisioned and accepts plain commands, with
//! a warning. An unreadable stored key **fails closed**: the device uses a random
//! in-memory key that no desktop knows (read-only over the radio) until a key is
//! provisioned again over USB.

use std::sync::{Mutex, RwLock};

use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use esp_idf_svc::sys;
use pinora_protocol::auth::{to_hex, Key, KEY_BYTES};

const NAMESPACE: &str = "pinora_auth";
const KEY_NAME: &str = "device_key";

/// The ESP32 base MAC as 12 lowercase hex digits.
pub fn device_id() -> String {
    let mut mac = [0u8; 6];
    // SAFETY: `mac` is a writable 6-byte buffer, as the function requires.
    let result = unsafe { sys::esp_efuse_mac_get_default(mac.as_mut_ptr()) };
    if result != sys::ESP_OK {
        log::error!("Could not read the base MAC address: {result}");
    }
    to_hex(&mac)
}

/// Bytes from the hardware RNG. ESP-IDF guarantees true randomness only while Wi-Fi or
/// Bluetooth is running, or while the bootloader entropy source is enabled.
pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    // SAFETY: the pointer and length describe `bytes`.
    unsafe { sys::esp_fill_random(bytes.as_mut_ptr().cast(), N) };
    bytes
}

/// Random bytes when the radio may be off: enables the SAR-ADC entropy source around
/// the read. Must not be used while Wi-Fi or Bluetooth is running (ESP-IDF rule).
fn random_bytes_radio_off<const N: usize>() -> [u8; N] {
    // SAFETY: called only while no radio is active (see `KeyStore`); enable and
    // disable are paired.
    unsafe { sys::bootloader_random_enable() };
    let bytes = random_bytes::<N>();
    unsafe { sys::bootloader_random_disable() };
    bytes
}

pub struct KeyStore {
    nvs: Mutex<EspNvs<NvsDefault>>,
    current: RwLock<Option<Key>>,
    device_id: String,
    radio_active: bool,
}

impl KeyStore {
    /// Opens the key store. Call before the radio starts. `radio_active` says whether
    /// Wi-Fi or Bluetooth will be running when keys are generated later.
    pub fn open(partition: EspDefaultNvsPartition, radio_active: bool) -> anyhow::Result<Self> {
        let nvs = EspNvs::new(partition, NAMESPACE, true)?;
        let mut buffer = [0u8; 64];
        let current = match nvs.get_blob(KEY_NAME, &mut buffer) {
            Ok(None) => None,
            Ok(Some(bytes)) if bytes.len() == KEY_BYTES => {
                let mut key = [0u8; KEY_BYTES];
                key.copy_from_slice(bytes);
                Some(Key::from_bytes(key))
            }
            Ok(Some(_)) | Err(_) => {
                log::error!(
                    "The stored device key is unreadable; radio commands stay rejected until a key is provisioned over USB"
                );
                Some(Key::from_bytes(random_bytes_radio_off()))
            }
        };
        let device_id = device_id();
        log::info!(
            "Device {device_id}: {}",
            if current.is_some() {
                "key provisioned; radio commands require authentication"
            } else {
                "no key; radio commands are not authenticated"
            }
        );
        Ok(Self {
            nvs: Mutex::new(nvs),
            current: RwLock::new(current),
            device_id,
            radio_active,
        })
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// The key that sessions must use, if any.
    pub fn current(&self) -> Option<Key> {
        self.current
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Returns the stored key, creating and storing one first if there is no valid
    /// stored key.
    pub fn provision(&self) -> anyhow::Result<Key> {
        let nvs = self
            .nvs
            .lock()
            .map_err(|_| anyhow::anyhow!("key store lock poisoned"))?;
        let mut buffer = [0u8; 64];
        if let Ok(Some(bytes)) = nvs.get_blob(KEY_NAME, &mut buffer) {
            if bytes.len() == KEY_BYTES {
                let mut key = [0u8; KEY_BYTES];
                key.copy_from_slice(bytes);
                return Ok(Key::from_bytes(key));
            }
        }
        let key = Key::from_bytes(if self.radio_active {
            random_bytes()
        } else {
            random_bytes_radio_off()
        });
        nvs.set_blob(KEY_NAME, key.as_bytes())?;
        *self
            .current
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(key.clone());
        Ok(key)
    }

    /// Deletes the key; the device becomes unprovisioned.
    pub fn clear(&self) -> anyhow::Result<()> {
        let nvs = self
            .nvs
            .lock()
            .map_err(|_| anyhow::anyhow!("key store lock poisoned"))?;
        nvs.remove(KEY_NAME)?;
        *self
            .current
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
        Ok(())
    }
}
