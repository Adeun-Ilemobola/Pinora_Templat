use std::sync::mpsc::SyncSender;

use crate::{
    core::{
        hardware::{OutputPinCore, TimerState},
        modulecore::{Module, ModuleCore},
    },
    protocol::{
        command::{ModuleCommand, RfidCommand},
        global_definitions::{ModuleType, RGBMode, RGB},
        module_event::{ModuleEvent, RfidEvent},
        registration::{ProtocolMessage, Registration},
    },
};

use esp_idf_svc::hal::{delay::Ets, spi::SpiSingleDeviceDriver};
use mfrc522::{
    comm::blocking::spi::{DummyDelay, SpiInterface},
    Initialized, Mfrc522, MifareKey,
};
use serde::{Deserialize, Serialize};

fn rfid_spi_delay() {
    Ets::delay_us(1);
}
type RfidDelay = fn();

type RfidReader<'d> = Mfrc522<RfidSpiInterface<'d>, Initialized>;
type RfidSpiInterface<'d> = SpiInterface<SpiSingleDeviceDriver<'d>, RfidDelay>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MddeRfid {
    Read,
    Write,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WriteState {
    Good,
    Bad,
}
const USABLE_BLOCKS: [u8; 47] = [
    1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, 16, 17, 18, 20, 21, 22, 24, 25, 26, 28, 29, 30, 32, 33,
    34, 36, 37, 38, 40, 41, 42, 44, 45, 46, 48, 49, 50, 52, 53, 54, 56, 57, 58, 60, 61, 62,
];
pub struct Rfid<'d> {
    pub core: ModuleCore,
    pub temp_read_data: Vec<u8>,
    pub temp_write_data: Vec<u8>,
    write_state: WriteState,
    write_state_msg: String,
    pub write_chuck: u32,
    pub next_acc: TimerState,
    pub mode: MddeRfid,

    mf: RfidReader<'d>,
    rgb_pins: RGB<'d>,
    buzer_pin: OutputPinCore<'d>,
    pub rgb_mode: RGBMode,
}

impl<'d> Rfid<'d> {
    pub fn new(
        spi: SpiSingleDeviceDriver<'d>,
        rgb_pins: RGB<'d>,
        buzer_pin: OutputPinCore<'d>,
        core_id: &str,
        parent_id: Option<String>,
        sender: SyncSender<ProtocolMessage>,
    ) -> anyhow::Result<Self> {
        let interface = SpiInterface::new(spi).with_delay(rfid_spi_delay as RfidDelay);

        let mut reader = Mfrc522::new(interface)
            .init()
            .map_err(|error| anyhow::anyhow!("failed to initialize MFRC522: {error:?}"))?;

        let version = reader
            .version()
            .map_err(|error| anyhow::anyhow!("failed to read MFRC522 version: {error:?}"))?;
        // anyhow::ensure!(
        //     matches!(version, 0x91 | 0x92),
        //     "MFRC522 did not respond with a valid version (got 0x{version:02X}); check 3.3 V, GND, RST, SCK, MOSI, MISO, and CS wiring"
        // );
        log::info!("MFRC522 detected (version 0x{version:02X})");

        let dor = Self {
            core: ModuleCore::new(ModuleType::Rfid, core_id, sender),
            temp_read_data: vec![],
            temp_write_data: vec![],
            next_acc: TimerState::from_ms(20.0),
            mode: MddeRfid::Read,
            mf: reader,
            rgb_pins,
            rgb_mode: RGBMode::Off,
            buzer_pin,
            write_chuck: 1,
            write_state: WriteState::Good,
            write_state_msg: String::new(),
        };

        dor.Registration(Registration {
            id: dor.id().to_string(),
            module_type: dor.get_module_type().clone(),
            lool_up_id: core_id.to_string(),
            parent_id: parent_id.clone().unwrap_or_default(),
        });

        Ok(dor)
    }

    pub fn tick(&mut self) -> anyhow::Result<(), ()> {
        if !self.next_acc.ready() {
            return Ok(());
        }

        if self.rgb_mode != RGBMode::Off {
            self.update_rgb(RGBMode::Off);
        }
        match self.mode {
            MddeRfid::Read => {
                match self.mf.new_card_present() {
                    Ok(atqa) => {
                        let uid = match self.mf.select(&atqa) {
                            Ok(d) => d,
                            Err(e) => {
                                return Err(());
                            }
                        };
                        self.update_rgb(RGBMode::Blue);
                        println!("Card UID: {:02X?}", uid.as_bytes());
                        let key: MifareKey = [0xFF; 6];
                        if !self.temp_read_data.is_empty() {
                            self.temp_read_data.clear();
                        }

                        for block in USABLE_BLOCKS.iter() {
                            match self.mf.mf_authenticate(&uid, *block, &key) {
                                Ok(_) => {
                                    let data: [u8; 16] =
                                        self.mf.mf_read(*block).map_err(|err| -> () {
                                            println!("data err ->{:?}", err)
                                        })?;

                                    self.temp_read_data.extend(data);

                                    self.mf
                                        .stop_crypto1()
                                        .map_err(|err| -> () { println!("{:?}", err) })?;
                                    let text = std::str::from_utf8(&self.temp_read_data).map_err(
                                        |err| -> () { println!(" to text map_err -> {:?}", err) },
                                    )?;
                                    self.emit(ModuleEvent::Rfid(RfidEvent::GetCard {
                                        id: self.id().to_string(),
                                        card_uid: format!("{:02X?}", uid.as_bytes()),
                                        card_data: text.to_string(),
                                    }));
                                }
                                Err(e) => {}
                            }
                        }

                        // Authenticate and read here when needed.
                    }

                    Err(mfrc522::Error::Timeout) => {
                        // No card currently present.
                    }

                    Err(error) => {
                        println!("RFID error: {error:?}");
                        return Err(());
                    }
                }
            }

            MddeRfid::Write => {
                if self.rgb_mode != RGBMode::Green {
                    self.update_rgb(RGBMode::Green);
                }
                match self.mf.new_card_present() {
                    Ok(atqa) => {
                        let uid = match self.mf.select(&atqa) {
                            Ok(d) => d,
                            Err(e) => {
                                return Err(());
                            }
                        };
                        self.update_rgb(RGBMode::Blue);
                        println!("Card UID: {:02X?}", uid.as_bytes());
                        let key: MifareKey = [0xFF; 6];
                        if self.temp_write_data.is_empty() {
                            self.mode = MddeRfid::Read;
                            self.emit(ModuleEvent::Rfid(RfidEvent::GetMode {
                                mode: self.mode.clone(),
                                 id: self.id().to_string(),
                            }));
                            return Ok(());
                        }
                        let max_chuck = self.temp_write_data.len().div_ceil(16);
                        let mut max_chuck_data = self.temp_write_data.chunks(16);

                        for (i, block) in USABLE_BLOCKS.iter().enumerate() {
                            match self.mf.mf_authenticate(&uid, *block, &key) {
                                Ok(_) => {
                                    match max_chuck_data.next() {
                                        Some(data) => {
                                            let mut block_data = [0u8; 16];
                                            block_data[..data.len()].copy_from_slice(data);
                                            match self.mf.mf_write(*block, block_data) {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    println!("mf_write  Err: {:?} ", e)
                                                }
                                            }
                                        }
                                        None => {
                                            self.write_state = WriteState::Bad;
                                            self.write_state_msg =
                                                "write data is is empty ".to_string();
                                            break;
                                        }
                                    }

                                    if (i + 1) >= max_chuck {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    self.write_state = WriteState::Bad;
                                    self.write_state_msg =
                                        format!("fiald to access {}", *block).to_string();
                                    break;
                                }
                            }
                        }

                        self.emit(ModuleEvent::Rfid(RfidEvent::GetWriteState {
                            id: self.id().to_string(),
                            state: self.write_state.clone(),
                            info: self.write_state_msg.to_string(),
                        }));

                        match self.write_state {
                            WriteState::Good => {
                                if self.rgb_mode != RGBMode::Red {
                                    self.update_rgb(RGBMode::Red);
                                }
                                self.mode = MddeRfid::Read;
                                self.emit(ModuleEvent::Rfid(RfidEvent::GetMode {
                                    mode: self.mode.clone(),
                                     id: self.id().to_string(),
                                }));
                            }
                            WriteState::Bad => {}
                        }

                        // Authenticate and read here when needed.
                    }

                    Err(mfrc522::Error::Timeout) => {
                        // No card currently present.
                    }

                    Err(error) => {
                        println!("RFID error: {error:?}");
                        return Err(());
                    }
                }
            }
        }
        Ok(())
    }

    fn update_rgb(&mut self, mode: RGBMode) {
        self.rgb_mode = mode;
        match self.rgb_mode {
            RGBMode::Blue => {
                let _ = self.rgb_pins.blue.set_state(true);
                let _ = self.rgb_pins.green.set_state(false);
                let _ = self.rgb_pins.red.set_state(false);
            }
            RGBMode::Green => {
                let _ = self.rgb_pins.blue.set_state(false);
                let _ = self.rgb_pins.green.set_state(true);
                let _ = self.rgb_pins.red.set_state(false);
            }
            RGBMode::Red => {
                let _ = self.rgb_pins.blue.set_state(false);
                let _ = self.rgb_pins.green.set_state(false);
                let _ = self.rgb_pins.red.set_state(true);
            }
            RGBMode::Off => {
                let _ = self.rgb_pins.blue.set_state(false);
                let _ = self.rgb_pins.green.set_state(false);
                let _ = self.rgb_pins.red.set_state(false);
            }
        }
    }
}

impl<'d> Module for Rfid<'d> {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Rfid(command) => match command {
                RfidCommand::ReadMode => {}
                RfidCommand::WriteMode => {}
                RfidCommand::WritePayload { data } => {
                    self.temp_write_data.extend(data);
                    self.mode = MddeRfid::Write;
                    self.emit(ModuleEvent::Rfid(RfidEvent::GetMode {
                        mode: self.mode.clone(),
                         id: self.id().to_string(),
                    }));
                }
            },

            _ => {
                // handle anything else
            }
        }
        Ok(())
    }
}
