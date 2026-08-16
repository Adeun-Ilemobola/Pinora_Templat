use std::time::{Duration, Instant};

use esp_idf_svc::hal::gpio::Level;

use crate::{
    core::{
        emitter::Emitter,
        hardware::{InputPinCore, TimerState},
        modulecore::{Module, ModuleCore},
    },
    protocol::{command::ModuleCommand, global_definitions::ModuleType},
};

pub struct RemoteReceiver<'d> {
    core: ModuleCore,
    state: Level,
    step_timer: TimerState,
    pin_driver: InputPinCore<'d>,
    test_time: Instant,
    buffer_raw: Vec<(Level, Duration)>,
    bits: Vec<u8>,
}

impl<'d> RemoteReceiver<'d> {
    pub fn new(
        pin: InputPinCore<'d>,
        core_id: String,
        sender: Emitter,
    ) -> Result<RemoteReceiver<'d>, ()> {
        let r = RemoteReceiver {
            core: ModuleCore::new(ModuleType::RemoteReceiver, &core_id, sender),
            pin_driver: pin,
            state: Level::Low,
            test_time: Instant::now(),
            buffer_raw: vec![],
            bits: vec![],
            step_timer: TimerState::from_ms(100.6),
        };
        Ok(r)
    }
    fn extract(&mut self) {
        let mut bits: Vec<u8> = Vec::new();
        let mut i = 3;
        while i + 1 < self.buffer_raw.len() {
            let low = self.buffer_raw[i];
            let high = self.buffer_raw[i + 1];

            // low should be around 560 us

            let bit = if high.1.as_micros() > 1000 { 1 } else { 0 };

            bits.push(bit);

            i += 2;
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

            println!(
                "decimal: {} | hex: 0x{:02X} | binary: {:08b}",
                value, value, value
            );
             data.push(value);
        }
       println!("Data: {:02X?}", data);
    }
}

impl<'d> Module for RemoteReceiver<'d> {
    fn tick(&mut self) -> Result<(), ()> {
        let data_now = self.pin_driver.now().unwrap();
        let previous_state = self.state;
        if data_now != self.state {
            let elapsed = self.test_time.elapsed();
            self.test_time = Instant::now();
            self.buffer_raw.push((previous_state, elapsed));
            self.state = data_now;
        }
        if self.step_timer.ready() && !self.buffer_raw.is_empty() {
            println!("-----------------Next-------------------");
            // for (index, bit) in self.buffer_raw.iter().enumerate() {
            //     println!("[{}] | {:?} ----- {:?}", index, bit.0, bit.1)
            // }
            self.extract();
            self.buffer_raw.clear();
            self.step_timer.reset();
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
