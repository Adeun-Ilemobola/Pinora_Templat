pub mod core;
pub mod module;
pub mod protocol;
pub mod utilities;

use crate::core::emitter::Emitter;
use crate::core::hardware::*;
use crate::core::modulecore::{Module};
use crate::module::stepper::{StepperMotor, StepperPinMode, StepperPins};
use crate::protocol::command::IncomingCommand;
use esp_idf_svc::hal::spi::{
    config::{Config as SpiConfig, DriverConfig, MODE_0},
    SpiDeviceDriver,
};
use std::io;
use std::io::{BufRead, ErrorKind};
use std::sync::mpsc;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

type ModuleHandle<'a> = Rc<RefCell<dyn Module + 'a>>;

fn configure_console_uart() -> anyhow::Result<()> {
    use esp_idf_svc::sys;
    use std::ptr;

    unsafe {
        let uart = sys::uart_port_t_UART_NUM_0;

        if !sys::uart_is_driver_installed(uart) {
            let result = sys::uart_driver_install(
                uart,
                2048, // RX buffer
                2048, // TX buffer
                0,    // no event queue
                ptr::null_mut(),
                0,
            );

            if result != sys::ESP_OK {
                anyhow::bail!("Failed to install UART driver: {}", result);
            }
        }

        sys::uart_vfs_dev_use_driver(uart as i32);
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let sync_sender = Emitter::new(None);

    configure_console_uart()?;
    print_esp_system_info(sync_sender.clone());
    let mut modules: HashMap<String, ModuleHandle<'_>> = HashMap::new();
    let p = Peripherals::take()?;
    let mut last_yield_us = now_us();
    // let i2c = I2cDriver::new(
    //     p.i2c0,
    //     p.pins.gpio21,
    //     p.pins.gpio22,
    //     &I2cConfig::new().baudrate(100.kHz().into()),
    // )?;
    let spi = SpiDeviceDriver::new_single(
        p.spi2,
        p.pins.gpio18,       //SCK, | -> purple
        p.pins.gpio19,       // MOSI |  -> blue
        Some(p.pins.gpio23), // MISO |  -> green
        Some(p.pins.gpio5),  // SDA / CS | -> gray
        &DriverConfig::new(),
        &SpiConfig::new().baudrate(1_000_000.Hz()).data_mode(MODE_0),
    )?;
    // MRC522 RST      -> GPIO 16

    //    LEFT                                      RIGHT
    //┌──────────────────────────────────────────────┐
    //│ SDA │ SCK │ MOSI │ MISO │ IRQ │ GND │ RST │ 3.3V │
    //└──────────────────────────────────────────────┘

    // let shared_i2c = Rc::new(RefCell::new(i2c));
    // let shared_spi = Rc::new(RefCell::new(spi));

    // let hardware = HardwareContext::new(p.ledc.timer0, shared_i2c.clone())?;
    // let rangefinder_i2c = RcDevice::new(hardware.i2c_bus.clone());

    // let lidar = Rc::new(RefCell::new(Lidar::new(
    //     hardware.servo_pwm.clone(),
    //     "lidar".to_string(),
    //     rangefinder_i2c,
    //     sync_sender.clone()
    // )?));
    // let lidar_id = lidar.borrow().get_id();
    // modules.insert(lidar_id, lidar.clone());

    let stepper = Rc::new(RefCell::new(StepperMotor::new(
        StepperPinMode::Manuel(StepperPins {
            in1: OutputPinCore::new(p.pins.gpio12)?, //33
            in2: OutputPinCore::new(p.pins.gpio14)?, //32
            in3: OutputPinCore::new(p.pins.gpio27)?, //31
            in4: OutputPinCore::new(p.pins.gpio26)?, //30
        }),
        "stepperX".to_string(),
        None,
        sync_sender.clone(),
    )?));

    modules.insert(stepper.borrow().id().to_owned(), stepper.clone());

    // const MPU_ADDRESS: u8 = 0x68;
    // let imu_i2c = RcDevice::new(shared_i2c.clone());
    // let mut  test_imu = MpuDevice::new(imu_i2c, MPU_ADDRESS ,sync_sender.clone() , "MPu" , None ).map_err(|err| anyhow::anyhow!("{err:?}"))?;

    // let   rfid = Rc::new(RefCell::new(
    //     Rfid::new(
    //     spi,
    //     RGB{
    //         red : OutputPinCore::new(p.pins.gpio12)?,
    //         green:  OutputPinCore::new(p.pins.gpio14)?,
    //         blue: OutputPinCore::new(p.pins.gpio27)? ,

    //     },
    //     OutputPinCore::new(p.pins.gpio17)?,
    //     "dff",
    //     None,
    //     sync_sender.clone()
    // ).map_err(|err| anyhow::anyhow!("{err:?}"))?
    // ));

    // modules.insert(rfid.borrow().id().to_owned(), rfid.clone());

    let (command_sender, command_receiver) = mpsc::channel::<IncomingCommand>();
    std::thread::spawn(move || {
        serial_command_reader(command_sender);
    });

    loop {
        // for module in modules.values() {
        //     let _ = module.borrow_mut().tick();
        // }
         let _ =stepper.borrow_mut().tick();

        if let Ok(command) = command_receiver.try_recv() {
            if let Some(module) = modules.get_mut(&command.id) {
                module.borrow_mut().handle_command(&command.command)?;
            } else {
                log::error!(
                    "No top-level module found for command id={} command={:?}",
                    command.id,
                    command.command
                );
            }
        }
        let now = now_us();
        if now - last_yield_us >= 650_000.0 {
            rtos_sleep_ms(1);
            last_yield_us = now_us();
        }
    }
}

fn serial_command_reader(command_sender: mpsc::Sender<IncomingCommand>) {
    let stdin = io::stdin();

    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(line) => line,
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                // Not an error — just no data yet. Back off and retry.
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            Err(err) => {
                log::error!("Real serial read error: {:?}", err);
                continue;
            }
        };

        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<IncomingCommand>(line) {
            Ok(command) => {
                log::info!("Parsed command: {:?}", command);

                let _ = command_sender.send(command);
            }

            Err(err) => {
                log::error!("Failed to parse command: {:?}", err);
                log::error!("Raw line was: {}", line);
            }
        }
    }
}
