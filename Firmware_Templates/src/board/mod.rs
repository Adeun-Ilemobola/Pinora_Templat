//! Board wiring for the current firmware image: which modules exist and which
//! pins and peripherals they own.
//!
//! # Pin map (current image)
//!
//! | Module | GPIO | LEDC channel | Notes |
//! |---|---|---|---|
//! | `led1` | 12 | 0 | GPIO12 (MTDI) is a boot strapping pin: if held high at reset it selects 1.8 V flash on most classic ESP32 modules (PIN-TPL-018) |
//! | `led2` | 14 | 1 | |
//! | `led3` | 27 | 2 | |
//! | I²C0 bus | SDA 21, SCL 22 | — | 100 kHz, created in `main.rs`; no consumer in this image |
//!
//! The same assignments are kept as data in [`pin_map::PIN_MAP`], which a host test
//! checks against `main.rs` (audit Phase 2E, PIN-TPL-018).
//!
//! All LEDs share LEDC timer0 (5 kHz, 13-bit). Wiring for the other module
//! families is kept, commented out, in [`disabled_wiring`]; read its warnings before
//! enabling any of it.

pub mod disabled_wiring;
pub mod pin_map;

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::hardware::{ledc, HardwareContext, OutputPin};
use crate::core::module::{Module, ModuleRegistry};
use crate::core::transport::emitter::TransportEmitter;
use crate::modules::actuators::led::LedModule;

/// Constructs the modules of this image and inserts them into `modules`, keyed by
/// their runtime UUID. Construction order is `led1`, `led2`, `led3`.
#[allow(clippy::too_many_arguments)]
pub fn install_active_modules<'d, P12, P14, P27, C0, C1, C2>(
    modules: &mut ModuleRegistry<'d>,
    shared: &Rc<RefCell<HardwareContext<'d>>>,
    gpio12: P12,
    gpio14: P14,
    gpio27: P27,
    channel0: C0,
    channel1: C1,
    channel2: C2,
    sync_sender: &TransportEmitter,
) -> anyhow::Result<()>
where
    P12: OutputPin + 'd,
    P14: OutputPin + 'd,
    P27: OutputPin + 'd,
    C0: ledc::LedcChannel<SpeedMode = ledc::LowSpeed> + 'd,
    C1: ledc::LedcChannel<SpeedMode = ledc::LowSpeed> + 'd,
    C2: ledc::LedcChannel<SpeedMode = ledc::LowSpeed> + 'd,
{
    let led1 = {
        let hardware = shared.borrow();

        LedModule::new(
            gpio12,
            channel0,
            "led1".to_string(),
            &hardware.led_timer,
            None,
            sync_sender.clone(),
        )?
    };
    let led1_id = led1.id().to_owned();
    modules.insert(led1_id, Box::new(led1));

    let led2 = {
        let hardware = shared.borrow();

        LedModule::new(
            gpio14,
            channel1,
            "led2".to_string(),
            &hardware.led_timer,
            None,
            sync_sender.clone(),
        )?
    };

    let led2_id = led2.id().to_owned();
    modules.insert(led2_id, Box::new(led2));

    let led3 = {
        let hardware = shared.borrow();

        LedModule::new(
            gpio27,
            channel2,
            "led3".to_string(),
            &hardware.led_timer,
            None,
            sync_sender.clone(),
        )?
    };

    let led3_id = led3.id().to_owned();
    modules.insert(led3_id, Box::new(led3));

    Ok(())
}
