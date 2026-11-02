//! UART0 console setup.

use esp_idf_svc::hal::uart::UART0;

/// Installs the UART0 driver (2 KiB RX/TX buffers) and routes stdio through it, so
/// the Serial transport can read commands from `stdin` and `println!` reaches the host.
///
/// Ownership (PIN-TPL-026, since audit Phase 3A): this drives UART0 through raw
/// ESP-IDF calls, so it takes `p.uart0` by value. The peripheral is then gone from
/// `Peripherals` and the type system prevents a second driver on UART0 (before 3A the
/// rule was only documented).
pub fn configure_console_uart(_uart0: UART0<'static>) -> anyhow::Result<()> {
    use esp_idf_svc::sys;
    use std::ptr;

    // SAFETY: UART0 exists on every ESP32; the driver is installed at most once
    // (guarded by `uart_is_driver_installed`) with no event queue.
    unsafe {
        let uart = sys::uart_port_t_UART_NUM_0;

        if !sys::uart_is_driver_installed(uart) {
            let result = sys::uart_driver_install(
                uart,
                2048, // RX buffer
                2048, // TX buffer
                0,    // no event queue
                ptr::null_mut(),
                0,
            );

            if result != sys::ESP_OK {
                anyhow::bail!("Failed to install UART driver: {}", result);
            }
        }

        sys::uart_vfs_dev_use_driver(uart as i32);
    }

    Ok(())
}
