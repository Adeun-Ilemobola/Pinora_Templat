use std::time::{Duration, Instant};

use esp_idf_svc::hal::gpio::Level;

use crate::core::{
    emitter::Emitter,
    hardware::InputPinCore,
    modulecore::{Module, ModuleCore},
};
use pinora_protocol::{
    command::ModuleCommand, global_definitions::ModuleType, ModuleEvent::RemoteReceiver,
    Registration, RemoteButton, RemoteButtonEvent,
};

pub struct RemoteReceiverButton<'d> {
    core: ModuleCore,
    state: Level,
    // invalidate: TimerState,
    pin_driver: InputPinCore<'d>,
    test_time: Instant,
    buffer_raw: Vec<(Level, Duration)>,
    ignore_initial: bool,
    remote_button: RemoteButton,
}

impl<'d> RemoteReceiverButton<'d> {
    pub fn new(
        pin: InputPinCore<'d>,
        core_id: String,
        sender: Emitter,
    ) -> Result<RemoteReceiverButton<'d>, ()> {
        let r = RemoteReceiverButton {
            core: ModuleCore::new(ModuleType::RemoteReceiver, &core_id, sender),
            pin_driver: pin,
            state: Level::Low,
            test_time: Instant::now(),
            buffer_raw: vec![],
            ignore_initial: true,

            remote_button: RemoteButton::None,
        };
        r.registration(Registration {
            id: r.id().to_string(),
            module_type: ModuleType::RemoteReceiver,
            lool_up_id: core_id,
            parent_id: String::new(),
        });
        Ok(r)
    }
    fn extract(&mut self) -> bool {
        let mut start_index = None;

        // Find NEC leader:
        // ~9ms LOW followed by ~4.5ms HIGH
        for i in 0..self.buffer_raw.len().saturating_sub(1) {
            let (level_a, time_a) = self.buffer_raw[i];
            let (level_b, time_b) = self.buffer_raw[i + 1];

            if level_a == Level::Low
                && (8_000..=10_000).contains(&time_a.as_micros())
                && level_b == Level::High
                && (3_500..=5_500).contains(&time_b.as_micros())
            {
                start_index = Some(i + 2);
                break;
            }
        }

        let Some(start) = start_index else {
            return false;
        };

        // 32 bits × 2 timing entries per bit
        if self.buffer_raw.len() < start + 64 {
            return false;
        }

        let mut data = [0u8; 4];

        for bit_index in 0..32 {
            let low = self.buffer_raw[start + bit_index * 2];
            let high = self.buffer_raw[start + bit_index * 2 + 1];

            if low.0 != Level::Low
                || !(350..=800).contains(&low.1.as_micros())
                || high.0 != Level::High
            {
                return false;
            }

            let bit = match high.1.as_micros() {
                350..=900 => 0,
                1_200..=2_200 => 1,
                _ => return false,
            };

            if bit == 1 {
                data[bit_index / 8] |= 1 << (bit_index % 8);
            }
        }

        // Validate NEC inverse bytes FIRST
        if data[0] ^ data[1] != 0xFF {
            return false;
        }

        if data[2] ^ data[3] != 0xFF {
            return false;
        }

        // Only print VALID packets
        // println!(
        //     "VALID COMMAND: decimal={} | hex=0x{:02X} | binary={:08b}",
        //     data[2], data[2], data[2]
        // );

        self.remote_button = RemoteButton::from_command(data[2]);
        self.emit(RemoteReceiver(RemoteButtonEvent::Click {
            id: self.id().to_string(),
            key: self.remote_button.clone(),
        }));
        self.remote_button = RemoteButton::None;
        true
    }
}

impl<'d> Module for RemoteReceiverButton<'d> {
    fn tick(&mut self) -> Result<(), ()> {
        if self.ignore_initial {
            self.ignore_initial = false;
            return Ok(());
        }

        let data_now = self.pin_driver.now().unwrap();
        let previous_state = self.state;
        if data_now != self.state {
            let elapsed = self.test_time.elapsed();
            self.test_time = Instant::now();
            self.buffer_raw.push((previous_state, elapsed));
            self.state = data_now;
        }

        if !self.buffer_raw.is_empty() && self.test_time.elapsed() > Duration::from_millis(15) {
            self.extract();
            self.buffer_raw.clear();
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
