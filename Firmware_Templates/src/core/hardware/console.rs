//! UART0 console setup.

/// Installs the UART0 driver (2 KiB RX/TX buffers) and routes stdio through it, so
/// the Serial transport can read commands from `stdin` and `println!` reaches the host.
///
/// Ownership note (PIN-TPL-026): this drives UART0 through raw ESP-IDF calls, while
/// `Peripherals::take()` still hands out `p.uart0` as if it were free. Never also
/// construct a `UartDriver` on `p.uart0`.
pub fn configure_console_uart() -> anyhow::Result<()> {
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
