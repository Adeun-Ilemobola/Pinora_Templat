use std::time::{Duration, Instant};

use esp_idf_svc::hal::gpio::Level;

use crate::{
    core::{
        emitter::Emitter,
        hardware::{InputPinCore, TimerState},
        modulecore::{Module, ModuleCore},
    },
};
use pinora_protocol::{command::ModuleCommand, global_definitions::ModuleType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteButton {
    Power,
    VolumeUp,
    FunctionStop,

    Previous,
    PlayPause,
    Next,

    Down,
    VolumeDown,
    Up,

    Zero,
    Equalizer,
    StopRepeat,

    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl RemoteButton {
    pub fn from_command(command: u8) -> Option<Self> {
        match command {
            0x45 => Some(Self::Power),
            0x46 => Some(Self::VolumeUp),
            0x47 => Some(Self::FunctionStop),

            0x44 => Some(Self::Previous),
            0x40 => Some(Self::PlayPause),
            0x43 => Some(Self::Next),

            0x07 => Some(Self::Down),
            0x15 => Some(Self::VolumeDown),
            0x09 => Some(Self::Up),

            0x19 => Some(Self::Equalizer),
            0x0D => Some(Self::StopRepeat),

            // Num
            0x16 => Some(Self::Zero),
            0x0C => Some(Self::One),
            0x18 => Some(Self::Two),
            0x5E => Some(Self::Three),
            0x08 => Some(Self::Four),
            0x1C => Some(Self::Five),
            0x5A => Some(Self::Six),
            0x42 => Some(Self::Seven),
            0x52 => Some(Self::Eight),
            0x4A => Some(Self::Nine),

            _ => None,
        }
    }
}

pub struct RemoteReceiver<'d> {
    core: ModuleCore,
    state: Level,
    step_timer: TimerState,
    // invalidate: TimerState,
    pin_driver: InputPinCore<'d>,
    test_time: Instant,
    buffer_raw: Vec<(Level, Duration)>,
    ignore_initial: bool,
    remote_button: Option<RemoteButton>,
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
            ignore_initial: true,
            step_timer: TimerState::from_ms(100.6),
          
            remote_button: None,
        };
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

            println!(
                "decimal: {} | hex: 0x{:02X} | binary: {:08b}",
                value, value, value
            );
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

impl<'d> Module for RemoteReceiver<'d> {
    fn tick(&mut self) -> Result<(), ()> {
        if self.ignore_initial {
            self.ignore_initial = false;
            return Ok(());
        }
        match self.remote_button {
            None=>{

            }
            Some(data)=>{
                 println!("Data: {:?}", data);
                 if self.step_timer.ready(){
                    self.remote_button = None;
                 }
            }
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
