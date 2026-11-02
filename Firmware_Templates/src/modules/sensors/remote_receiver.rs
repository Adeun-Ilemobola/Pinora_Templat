//! NEC infrared remote receiver.
//!
//! Since audit Phase 2C (PIN-TPL-009, PIN-TPL-019) pulses are timed by the RMT
//! peripheral in hardware (1 µs resolution) instead of by sampling the GPIO from the
//! main loop, which needs sub-350 µs loop iterations and cannot work with a scheduled
//! loop. `tick()` only polls the RMT driver without blocking and decodes finished
//! frames with the pure [`super::nec`] decoder. Reading the pin no longer panics.
//!
//! Not instantiated in the current image, and not verified on hardware (no receiver
//! is attached).

use esp_idf_svc::hal::gpio::InputPin;
use esp_idf_svc::hal::rmt::{
    config::{ReceiveConfig, RxChannelConfig},
    PinState, RxChannelDriver, Symbol,
};
use std::time::Duration;

use crate::core::{
    module::{Module, ModuleCore, ModuleError},
    transport::emitter::TransportEmitter,
};
use crate::modules::sensors::nec;
use pinora_protocol::{
    command::ModuleCommand, global_definitions::ModuleType, ModuleEvent::RemoteReceiver,
    RemoteButton, RemoteButtonEvent,
};

/// One NEC frame is 67 pulses (34 symbols); room for a repeat code as well.
const MAX_SYMBOLS: usize = 64;

pub struct RemoteReceiverButton<'d> {
    core: ModuleCore,
    rx: RxChannelDriver<'d>,
    receive: ReceiveConfig,
    symbols: Vec<Symbol>,
}

impl<'d> RemoteReceiverButton<'d> {
    pub fn new(
        pin: impl InputPin + 'd,
        core_id: String,
        sender: TransportEmitter,
    ) -> anyhow::Result<RemoteReceiverButton<'d>> {
        // 1 MHz: one RMT tick per microsecond.
        let rx = RxChannelDriver::new(pin, &RxChannelConfig::default())?;
        let receive = ReceiveConfig {
            // Shorter pulses are glitches; ESP32 hardware filters up to ~3 µs.
            signal_range_min: Duration::from_nanos(3_000),
            // A high level longer than the 9 ms leader ends the frame.
            signal_range_max: Duration::from_millis(12),
            // Poll: never block the main loop.
            timeout: Some(0),
            ..Default::default()
        };
        Ok(RemoteReceiverButton {
            core: ModuleCore::new(ModuleType::RemoteReceiver, &core_id, None, sender),
            rx,
            receive,
            symbols: vec![Symbol::default(); MAX_SYMBOLS],
        })
    }
}

fn pulses(symbols: &[Symbol]) -> Vec<nec::Pulse> {
    symbols
        .iter()
        .flat_map(|symbol| [symbol.level0(), symbol.level1()])
        .filter(|pulse| pulse.ticks.ticks() > 0)
        .map(|pulse| {
            (
                pulse.pin_state == PinState::High,
                u32::from(pulse.ticks.ticks()),
            )
        })
        .collect()
}

impl<'d> Module for RemoteReceiverButton<'d> {
    fn tick(&mut self) -> Result<(), ModuleError> {
        // A timeout means no frame has finished yet; the capture keeps running.
        let Ok(count) = self.rx.receive(&mut self.symbols, &self.receive) else {
            return Ok(());
        };
        if let Some(command) = nec::decode(&pulses(&self.symbols[..count.min(MAX_SYMBOLS)])) {
            self.emit(RemoteReceiver(RemoteButtonEvent::Click {
                key: RemoteButton::from_command(command),
            }));
        }
        Ok(())
    }
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, _command: &ModuleCommand) -> anyhow::Result<()> {
        Ok(())
    }
}
