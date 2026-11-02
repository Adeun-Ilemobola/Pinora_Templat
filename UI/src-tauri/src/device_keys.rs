//! Keys of the devices this desktop has paired with (audit Phase 2B, PIN-TPL-003).
//!
//! Stored in `<app data dir>/device-keys.json` as `{"devices":{"<device id>":"<hex key>"}}`.
//! Keys stay in the Rust backend: no IPC command or event ever carries one to the
//! WebView. The file is plain JSON in the user's own app data directory; an OS
//! keychain would be stronger but needs a new dependency (not in 2B's scope).

use std::collections::BTreeMap;
use std::path::PathBuf;

use pinora_protocol::auth::{from_hex, Key};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct KeyFile {
    devices: BTreeMap<String, String>,
}

#[derive(Debug, Default)]
pub(crate) struct DeviceKeys {
    /// `None`: in memory only (tests, or no app data directory).
    path: Option<PathBuf>,
    file: KeyFile,
}

/// Device ids are the ESP32 base MAC as 12 hex digits.
pub(crate) fn is_device_id(text: &str) -> bool {
    from_hex::<6>(text).is_some()
}

impl DeviceKeys {
    /// Loads the file; a missing file is empty. An unreadable file is logged and
    /// treated as empty (and is overwritten by the next pairing).
    pub(crate) fn load(path: PathBuf) -> Self {
        let file = match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|error| {
                log::warn!("Ignoring unreadable {}: {error}", path.display());
                KeyFile::default()
            }),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => KeyFile::default(),
            Err(error) => {
                log::warn!("Could not read {}: {error}", path.display());
                KeyFile::default()
            }
        };
        Self {
            path: Some(path),
            file,
        }
    }

    pub(crate) fn get(&self, device_id: &str) -> Option<Key> {
        self.file
            .devices
            .get(device_id)
            .and_then(|hex| Key::from_hex(hex))
    }

    pub(crate) fn device_ids(&self) -> Vec<String> {
        self.file.devices.keys().cloned().collect()
    }

    pub(crate) fn insert(&mut self, device_id: &str, key: &Key) -> Result<(), String> {
        if !is_device_id(device_id) {
            return Err(format!("Invalid device id {device_id:?}"));
        }
        self.file
            .devices
            .insert(device_id.to_string(), key.to_hex());
        self.save()
    }

    /// Returns whether the device was known.
    pub(crate) fn remove(&mut self, device_id: &str) -> Result<bool, String> {
        let known = self.file.devices.remove(device_id).is_some();
        if known {
            self.save()?;
        }
        Ok(known)
    }

    fn save(&self) -> Result<(), String> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
        }
        let json = serde_json::to_vec_pretty(&self.file).map_err(|error| error.to_string())?;
        // Write a sibling file, then rename, so a crash never leaves a truncated file.
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, json).map_err(|error| error.to_string())?;
        std::fs::rename(&temp, path).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pinora_protocol::auth::KEY_BYTES;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("pinora-keys-test-{}-{name}", std::process::id()))
            .join("device-keys.json")
    }

    #[test]
    fn keys_persist_across_loads_and_can_be_forgotten() {
        let path = temp_path("persist");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
        let key = Key::from_bytes([7; KEY_BYTES]);

        let mut keys = DeviceKeys::load(path.clone());
        assert!(keys.get("a1b2c3d4e5f6").is_none());
        keys.insert("a1b2c3d4e5f6", &key).unwrap();

        let mut reloaded = DeviceKeys::load(path.clone());
        assert_eq!(reloaded.get("a1b2c3d4e5f6"), Some(key));
        assert_eq!(reloaded.device_ids(), vec!["a1b2c3d4e5f6".to_string()]);
        assert!(reloaded.remove("a1b2c3d4e5f6").unwrap());
        assert!(!reloaded.remove("a1b2c3d4e5f6").unwrap());
        assert!(DeviceKeys::load(path.clone()).get("a1b2c3d4e5f6").is_none());
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn a_corrupt_file_is_treated_as_empty() {
        let path = temp_path("corrupt");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"{not json").unwrap();
        assert!(DeviceKeys::load(path.clone()).device_ids().is_empty());
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn device_ids_are_validated() {
        let mut keys = DeviceKeys::default();
        let key = Key::from_bytes([1; KEY_BYTES]);
        assert!(keys.insert("../../etc", &key).is_err());
        assert!(keys.insert("a1b2c3d4e5f6", &key).is_ok());
    }
}
