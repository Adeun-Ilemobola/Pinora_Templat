//! The one-off `System` snapshot sent at startup (heap, flash, app slot, IDF version).
//!
//! Values are formatted strings captured once; they are not refreshed later.
//! `maximum_app_slot` is the size of the *running* partition, not a scan of all slots.

use pinora_protocol::registration::{SystemInfo, PROTOCOL_VERSION};
use std::{ffi::CStr, ptr};

use esp_idf_svc::sys;

use crate::core::transport::emitter::{EmitterError, TransportEmitter};

/// Sends the snapshot with `emit_reliable` (blocks until queued), so it is cached for
/// reconnect replay by the Bluetooth and Wi-Fi transports.
pub fn print_esp_system_info(emit: TransportEmitter) -> Result<(), EmitterError> {
    // SAFETY: read-only ESP-IDF queries with valid arguments; the partition pointer
    // is null-checked before it is dereferenced, and the IDF version string is static.
    unsafe {
        // -------------------------
        // RAM / heap information
        // -------------------------
        let memory_caps = sys::MALLOC_CAP_8BIT as u32;

        let total_heap = sys::heap_caps_get_total_size(memory_caps);

        let free_heap = sys::heap_caps_get_free_size(memory_caps);

        let minimum_free_heap = sys::heap_caps_get_minimum_free_size(memory_caps);

        let largest_free_block = sys::heap_caps_get_largest_free_block(memory_caps);

        // -------------------------
        // Flash size
        // -------------------------
        let mut flash_size = 0_u32;

        let flash_result = sys::esp_flash_get_size(ptr::null_mut(), &mut flash_size);

        // -------------------------
        // Maximum firmware/app slot
        // -------------------------
        let running_partition = sys::esp_ota_get_running_partition();

        let app_partition_size = if running_partition.is_null() {
            0
        } else {
            (*running_partition).size
        };

        // -------------------------
        // ESP-IDF version
        // -------------------------
        let idf_version = CStr::from_ptr(sys::esp_get_idf_version()).to_string_lossy();
        let flash = if flash_result == sys::ESP_OK {
            format_bytes(flash_size as usize)
        } else {
            "Failed to read".to_string()
        };

        emit.system_info(SystemInfo {
            protocol_version: PROTOCOL_VERSION,
            esp_idf_version: idf_version.to_string(),
            total_heap: format_bytes(total_heap),
            current_free_heap: format_bytes(free_heap),
            lowest_free_heap: format_bytes(minimum_free_heap),
            largest_allocation: format_bytes(largest_free_block),
            maximum_app_slot: format_bytes(app_partition_size as usize),
            flash,
        })
    }
}

fn format_bytes(bytes: usize) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;

    if bytes as f64 >= MIB {
        std::format!("{:.2} MiB ({bytes} bytes)", bytes as f64 / MIB)
    } else {
        std::format!("{:.2} KiB ({bytes} bytes)", bytes as f64 / KIB)
    }
}
