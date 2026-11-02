//! MFRC522 RFID reader/writer with a status RGB LED.
//!
//! Since audit Phase 2C (PIN-TPL-019): cards use the length-prefixed
//! [`super::rfid_record`] format (legacy 16-byte UUID cards still read), the write
//! buffer is bounded, and start-up fails when no known MFRC522 answers. Not
//! instantiated in the current image and not verified on hardware.

use crate::core::{
    hardware::{OutputPinCore, TimerState},
    module::{Module, ModuleCore, ModuleError},
    transport::emitter::{EmitterError, TransportEmitter},
};
use crate::modules::sensors::rfid_record::{self, Block, WriteBuffer, USABLE_BLOCKS};
use pinora_protocol::{
    command::ModuleCommand,
    global_definitions::ModuleType,
    module_event::{LogPriority, ModuleEvent, SysLogEvent},
};

use esp_idf_svc::hal::{delay::Ets, spi::SpiSingleDeviceDriver};
use mfrc522::{comm::blocking::spi::SpiInterface, Initialized, Mfrc522, MifareKey};
use serde::{Deserialize, Serialize};

pub use pinora_protocol::module::rfid::{MddeRfid, RfidCommand, RfidEvent, WriteState};

pub struct RGB<'d> {
    pub red: OutputPinCore<'d>,
    pub green: OutputPinCore<'d>,
    pub blue: OutputPinCore<'d>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RGBMode {
    Red,
    Green,
    Blue,
    Off,
}

fn rfid_spi_delay() {
    Ets::delay_us(1);
}
type RfidDelay = fn();

type RfidReader<'d> = Mfrc522<RfidSpiInterface<'d>, Initialized>;
type RfidSpiInterface<'d> = SpiInterface<SpiSingleDeviceDriver<'d>, RfidDelay>;

pub struct Rfid<'d> {
    pub core: ModuleCore,
    pub temp_read_data: Vec<Block>,
    pub temp_write_data: WriteBuffer,
    write_state: WriteState,
    write_state_msg: String,
    pub write_chunk: u32,
    pub next_acc: TimerState,
    pub mode: MddeRfid,

    mf: RfidReader<'d>,
    rgb_pins: RGB<'d>,
    buzzer_pin: OutputPinCore<'d>,
    pub rgb_mode: RGBMode,
}

impl<'d> Rfid<'d> {
    pub fn new(
        spi: SpiSingleDeviceDriver<'d>,
        rgb_pins: RGB<'d>,
        buzzer_pin: OutputPinCore<'d>,
        core_id: &str,
        parent_id: Option<String>,
        sender: TransportEmitter,
    ) -> anyhow::Result<Self> {
        let interface = SpiInterface::new(spi).with_delay(rfid_spi_delay as RfidDelay);

        let mut reader = Mfrc522::new(interface)
            .init()
            .map_err(|error| anyhow::anyhow!("failed to initialize MFRC522: {error:?}"))?;

        let version = reader
            .version()
            .map_err(|error| anyhow::anyhow!("failed to read MFRC522 version: {error:?}"))?;
        anyhow::ensure!(
            rfid_record::is_known_reader_version(version),
            "MFRC522 did not respond with a valid version (got 0x{version:02X}); check 3.3 V, GND, RST, SCK, MOSI, MISO, and CS wiring"
        );
        log::info!("MFRC522 detected (version 0x{version:02X})");

        let dor = Self {
            core: ModuleCore::new(ModuleType::Rfid, core_id, parent_id, sender),
            temp_read_data: vec![],
            temp_write_data: WriteBuffer::default(),
            next_acc: TimerState::from_ms(20.0),
            mode: MddeRfid::Read,
            mf: reader,
            rgb_pins,
            rgb_mode: RGBMode::Off,
            buzzer_pin,
            write_chunk: 1,
            write_state: WriteState::Good,
            write_state_msg: "All Good".to_string(),
        };

        Ok(dor)
    }

    fn tick_inner(&mut self) -> Result<(), ()> {
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
                                self.emit(ModuleEvent::SysLog(SysLogEvent {
                                    text: "mf.select".to_string(),
                                    priority: LogPriority::Critical,
                                    raw_err: Some(format!("{:?} ", e)),
                                }));
                                self.update_rgb(RGBMode::Red);
                                return Err(());
                            }
                        };
                        // let _ =self.buzzer_pin.set_state(true);
                        // let _ =self.buzzer_pin.set_state(false);
                        self.update_rgb(RGBMode::Blue);

                        self.read_card(&uid)?;
                        self.send_card_data(&uid)?;

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
                                self.emit(ModuleEvent::SysLog(SysLogEvent {
                                    text: "mf.select in Write".to_string(),
                                    priority: LogPriority::Critical,
                                    raw_err: Some(format!("{:?} ", e)),
                                }));
                                self.update_rgb(RGBMode::Red);
                                return Err(());
                            }
                        };
                        self.update_rgb(RGBMode::Blue);
                        // let _ =self.buzzer_pin.set_state(true);
                        // let _ =self.buzzer_pin.set_state(false);
                        // println!("Card UID: {:02X?}", uid.as_bytes());
                        if self.temp_write_data.is_empty() {
                            self.mode = MddeRfid::Read;
                            self.emit(ModuleEvent::Rfid(RfidEvent::GetMode {
                                mode: self.mode.clone(),
                            }));
                            return Ok(());
                        }

                        self.write_card(&uid)?;
                        self.read_card(&uid)?;
                        self.send_card_data(&uid)?;

                        self.emit(ModuleEvent::Rfid(RfidEvent::GetWriteState {
                            state: self.write_state.clone(),
                            info: self.write_state_msg.to_string(),
                        }));

                        match self.write_state {
                            WriteState::Good => {
                                if self.rgb_mode != RGBMode::Off {
                                    self.update_rgb(RGBMode::Off);
                                }
                                self.mode = MddeRfid::Read;
                                self.temp_write_data.clear();
                            }
                            WriteState::Bad => {
                                if self.rgb_mode != RGBMode::Red {
                                    self.update_rgb(RGBMode::Red);
                                }
                                self.mode = MddeRfid::Read;
                            }
                        }
                        self.emit(ModuleEvent::Rfid(RfidEvent::GetMode {
                            mode: self.mode.clone(),
                        }));

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

    fn read_card(&mut self, uid: &mfrc522::Uid) -> Result<(), ()> {
        let key: MifareKey = [0xFF; 6];
        self.temp_read_data.clear();
        // The first block says how many blocks the record spans.
        let mut wanted = 1;

        for block in USABLE_BLOCKS.iter() {
            if self.temp_read_data.len() >= wanted {
                break;
            }
            match self.mf.mf_authenticate(&uid, *block, &key) {
                Ok(_) => {
                    let data: [u8; 16] = self.mf.mf_read(*block).map_err(|err| -> () {
                        self.emit(ModuleEvent::SysLog(SysLogEvent {
                            text: format!("faild to mf_read  block: {:?}", *block),
                            priority: LogPriority::Critical,
                            raw_err: Some(format!("{:?}", err)),
                        }));
                    })?;

                    if self.temp_read_data.is_empty() {
                        wanted = rfid_record::blocks_to_read(&data);
                    }
                    self.temp_read_data.push(data);

                    self.mf.stop_crypto1().map_err(|err| -> () {
                        self.emit(ModuleEvent::SysLog(SysLogEvent {
                            text: format!("faild to stop_crypto1  block: {:?}", *block),
                            priority: LogPriority::Critical,
                            raw_err: Some(format!("{:?}", err)),
                        }));
                    })?;
                }
                Err(e) => {
                    self.emit(ModuleEvent::SysLog(SysLogEvent {
                        text: format!("faild to read block: {:?}", *block),
                        priority: LogPriority::Low,
                        raw_err: Some(format!("{:?}", e)),
                    }));
                }
            }
        }
        Ok(())
    }
    fn send_card_data(&mut self, uid: &mfrc522::Uid) -> Result<(), ()> {
        let data = match rfid_record::decode(&self.temp_read_data) {
            Ok(data) => data,
            Err(error) => {
                self.emit(ModuleEvent::SysLog(SysLogEvent {
                    text: format!("Unreadable card data: {error}"),
                    priority: LogPriority::High,
                    raw_err: None,
                }));
                return Err(());
            }
        };

        self.emit(ModuleEvent::Rfid(RfidEvent::GetCard {
            card_uid: format!("{:02X?}", uid.as_bytes()),
            card_data: rfid_record::render(&data),
        }));

        Ok(())
    }
    fn write_card(&mut self, uid: &mfrc522::Uid) -> Result<(), ()> {
        let key: MifareKey = [0xFF; 6];

        let blocks = match rfid_record::encode(self.temp_write_data.as_slice()) {
            Ok(blocks) => blocks,
            Err(error) => {
                self.write_state = WriteState::Bad;
                self.write_state_msg = error.to_string();
                return Ok(());
            }
        };
        let max_chuck = blocks.len();
        let mut max_chuck_data = blocks.iter();

        for (i, block) in USABLE_BLOCKS.iter().enumerate() {
            match self.mf.mf_authenticate(&uid, *block, &key) {
                Ok(_) => {
                    match max_chuck_data.next() {
                        Some(block_data) => {
                            match self.mf.mf_write(*block, *block_data) {
                                Ok(_) => {}
                                Err(e) => {
                                    // println!("mf_write  Err: {:?} ", e)
                                    self.write_state = WriteState::Bad;
                                    self.write_state_msg = format!("mf_write  Err: {:?} ", e);
                                    self.emit(ModuleEvent::SysLog(SysLogEvent {
                                        text: "mf_write".to_string(),
                                        priority: LogPriority::Critical,
                                        raw_err: Some(format!("{:?} ", e)),
                                    }));
                                    // break;
                                }
                            }
                        }
                        None => {
                            self.write_state = WriteState::Bad;
                            self.write_state_msg = "write data is is empty ".to_string();
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
                        format!("fiald to access {} | Err : {:?}", *block, e).to_string();

                    self.emit(ModuleEvent::SysLog(SysLogEvent {
                        text: format!("fiald to access {} ", *block,).to_string(),
                        priority: LogPriority::Critical,
                        raw_err: Some(format!("{:?} ", e)),
                    }));
                    break;
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
    fn tick(&mut self) -> Result<(), ModuleError> {
        self.tick_inner().map_err(ModuleError::from)
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::Rfid(command) => match command {
                RfidCommand::ReadMode {} => {
                    self.mode = MddeRfid::Read;
                }
                RfidCommand::WriteMode {} => {
                    self.mode = MddeRfid::Write;
                }
                RfidCommand::WritePayload { data } => {
                    // Bounded (PIN-TPL-019): an oversized payload is rejected, and the
                    // dispatcher reports the error.
                    self.temp_write_data.append(data)?;
                    self.mode = MddeRfid::Write;
                    self.emit(ModuleEvent::Rfid(RfidEvent::GetMode {
                        mode: self.mode.clone(),
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
