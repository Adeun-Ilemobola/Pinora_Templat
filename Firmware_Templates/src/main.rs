//! Pinora firmware entry point.
//!
//! Startup order is part of the firmware's behavior (contract C-15). Keep it:
//! patches and logger → peripherals → console UART → event loop and NVS → transport
//! (one per image; since audit Phase 2B it first opens the NVS key store and then
//! starts the USB console reader) → system snapshot → I²C bus → shared hardware →
//! modules → registration → command receiver → main loop.
//!
//! Locals declared here (transport, buses, modules) live until `main` returns, which
//! only happens on an error.

pub mod board;
pub mod core;
pub mod modules;
pub mod runtime;
pub mod utilities;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use crate::core::hardware::*;
use crate::core::module::ModuleRegistry;
use crate::core::transport::manager::{TransportCore, TransportType};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let p = Peripherals::take()?;

    configure_console_uart()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    // Select one transport for this firmware image: Serial, Wifi, or Bluetooth.
    let selected_transport = TransportType::Bluetooth;
    let transport = TransportCore::new(selected_transport, sys_loop.clone(), nvs.clone(), p.modem)?;
    let sync_sender = transport.emitter.clone();

    print_esp_system_info(sync_sender.clone())?;
    let mut modules: ModuleRegistry<'_> = HashMap::new();
    let last_yield_us = now_us();

    let i2c = I2cDriver::new(
        p.i2c0,
        p.pins.gpio21,
        p.pins.gpio22,
        &I2cConfig::new().baudrate(100.kHz().into()),
    )?;

    let shared_i2c = Rc::new(RefCell::new(i2c));

    let hardware = HardwareContext::new(p.ledc.timer0, shared_i2c.clone())?;
    let shared = Rc::new(RefCell::new(hardware));
    // Wiring for inactive module families: see `board::disabled_wiring`.

    board::install_active_modules(
        &mut modules,
        &shared,
        p.pins.gpio12,
        p.pins.gpio14,
        p.pins.gpio27,
        p.ledc.channel0,
        p.ledc.channel1,
        p.ledc.channel2,
        &sync_sender,
    )?;

    runtime::register_all(&mut modules)?;

    let command_receiver = transport.handle_incoming()?;

    runtime::run(&mut modules, command_receiver, last_yield_us, sync_sender)
}
