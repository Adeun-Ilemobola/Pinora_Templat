//! Hardware helpers shared by every module: GPIO wrappers, timing, shared-bus
//! handles, the console UART and the startup system snapshot.
//!
//! Everything here lives on the main task. The shared buses are `Rc<RefCell<…>>`
//! (deliberately not `Send`), so modules cannot be moved to other threads and all
//! peripheral access stays single-threaded. Keep that invariant: transports talk to
//! modules only through channels.
//!
//! The `pub use` re-exports below keep the historical `crate::core::hardware::*`
//! import paths working for modules and `main.rs`.

mod console;
mod context;
mod pins;
mod system_info;
mod timing;

pub use console::configure_console_uart;
pub use context::*;
pub use pins::*;
pub use system_info::print_esp_system_info;
pub use timing::*;

pub use esp_idf_svc::hal::delay::FreeRtos;
pub use esp_idf_svc::hal::gpio::*;
pub use esp_idf_svc::hal::i2c;
pub use esp_idf_svc::hal::i2c::{I2c, I2cConfig, I2cDriver, I2cError};
pub use esp_idf_svc::hal::ledc;
pub use esp_idf_svc::hal::peripherals::Peripherals;
pub use esp_idf_svc::hal::uart::UartDriver;
pub use esp_idf_svc::hal::units::*;
pub use esp_idf_svc::partition::*;
