pub mod core;
pub mod module;
pub mod protocol;
pub mod utilities;
use embedded_hal_bus::i2c::RcDevice;

use crate::core::hardware::*;
use crate::core::modulecore::Module;
use crate::module::joystick::JoyStick;
use crate::module::lidar::Lidar;
use crate::protocol::command::IncomingCommand;
// use crate::utilities::serdeprotocol::IncomingCommand;

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
    configure_console_uart()?;
    let mut modules: HashMap<String, ModuleHandle<'_>> = HashMap::new();
    let p = Peripherals::take()?;
    let i2c = I2cDriver::new(
        p.i2c0,
        p.pins.gpio21,
        p.pins.gpio22,
        &I2cConfig::new().baudrate(400.kHz().into()),
    )?;

    let shared_i2c = Rc::new(RefCell::new(i2c));
    

    let hardware = HardwareContext::new(p.ledc.timer0,shared_i2c.clone())?;
    // let rangefinder_i2c = RcDevice::new(hardware.i2c_bus.clone());

    // let rangefinder = Rc::new(RefCell::new(Rangefinder::new(
    //     p.i2c1,
    //     p.pins.gpio21,
    //     p.pins.gpio22,
    //     "rangefinder".to_string(),
    //     None,
    // )?));
    // let rangefinder_id = rangefinder.borrow().id().clone();
    // modules.insert(rangefinder_id, rangefinder.clone());
    // print_welcome_message("not initialized", "not initialized");

    // let lidar = Rc::new(RefCell::new(Lidar::new(
    //     hardware.servo_pwm.clone(),
    //     "lidar".to_string(),
    //     rangefinder_i2c
    // )?));
    // let lidar_id = lidar.borrow().get_id();
    // modules.insert(lidar_id, lidar.clone());

    let mut joystick = JoyStick::new(
        p.pins.gpio25,
        p.adc1,
        p.pins.gpio34,
        p.pins.gpio35,
    )?;

    let (command_sender, command_receiver) = mpsc::channel::<IncomingCommand>();
    std::thread::spawn(move || {
        serial_command_reader(command_sender);
    });

    loop {
        // rangefinder.borrow_mut().tick();
        // lidar.borrow_mut().tick();
        joystick.tick()?;

        // if btu.poll()? {
        //     led_module.borrow_mut().toggle()?
        // }

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
        sleep_ms(10);
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
