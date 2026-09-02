use std::time::{Duration, Instant};

use esp_idf_svc::hal::gpio::Level;

use crate::core::{
    emitter::Emitter,
    hardware::{InputPinCore, TimerState},
    modulecore::{Module, ModuleCore},
};
use pinora_protocol::{
    command::ModuleCommand,
    global_definitions::ModuleType,
    ModuleEvent::{ RemoteReceiver},
    Registration, RemoteButton, RemoteButtonEvent,
};

pub struct RemoteReceiverButton<'d> {
    core: ModuleCore,
    state: Level,
    step_timer: TimerState,
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
            step_timer: TimerState::from_ms(100.6),

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
    fn extract(&mut self) {
        let mut bits: Vec<u8> = Vec::new();
        // let mut i = 3;

        let mut start_index = None;

        for i in 0..self.buffer_raw.len() - 1 {
            let (level_a, time_a) = self.buffer_raw[i];
            let (level_b, time_b) = self.buffer_raw[i + 1];

            if level_a == Level::Low
                && time_a.as_micros() > 8_000
                && time_a.as_micros() < 10_000
                && level_b == Level::High
                && time_b.as_micros() > 4_000
                && time_b.as_micros() < 5_000
            {
                start_index = Some(i + 2);
                break;
            }
        }
        match start_index {
            Some(mut i) => {
                while i + 1 < self.buffer_raw.len() {
                    let low = self.buffer_raw[i];
                    let high = self.buffer_raw[i + 1];

                    // low should be around 560 us

                    let bit = if high.1.as_micros() > 1000 { 1 } else { 0 };

                    bits.push(bit);

                    i += 2;
                }
            }
            None => {
                return;
            }
        }
        if bits.len() != 32 {
            return;
        }
        let mut data: Vec<u8> = Vec::new();

        for chunk in bits.chunks(8) {
            let mut value: u8 = 0;

            for (bit_index, bit) in chunk.iter().enumerate() {
                if *bit == 1 {
                    value |= 1 << bit_index;
                }
            }

            // println!(
            //     "decimal: {} | hex: 0x{:02X} | binary: {:08b}",
            //     value, value, value
            // );
            data.push(value);
        }

        if data[0] ^ data[1] != 0xFF {
            return;
        }

        if data[2] ^ data[3] != 0xFF {
            return;
        }
        self.remote_button = RemoteButton::from_command(data[2]);
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

        if self.step_timer.ready() && self.buffer_raw.len() > 2 {
            // println!(
            //     "---------------- FRAME len={} ----------------",
            //     self.buffer_raw.len()
            // );

            // for (index, item) in self.buffer_raw.iter().enumerate() {
            //     println!("[{}] {:?} - {} us", index, item.0, item.1.as_micros());
            // }
            self.extract();
            self.buffer_raw.clear();
            self.step_timer.reset();
        }

        if self.remote_button != RemoteButton::None {
            if self.step_timer.ready() {
                self.emit(RemoteReceiver(RemoteButtonEvent::Click {
                    id: self.id().to_string(),
                    key: self.remote_button.clone(),
                }));
                self.remote_button = RemoteButton::None;
            }
        }

        Ok(())
    }
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn get_module_type(&self) -> &ModuleType {
        &self.core.module_type
    }
    fn handle_command(&mut self, _command: &ModuleCommand) -> anyhow::Result<()> {
        Ok(())
    }
}
