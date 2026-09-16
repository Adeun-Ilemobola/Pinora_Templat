pub mod core;
pub mod module;
pub mod utilities;
use embedded_hal_bus::i2c::RcDevice;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use pinora_protocol::ServoCapability;
use pwm_pca9685::Channel;

use crate::core::hardware::*;
use crate::core::modulecore::Module;
use crate::core::transport::transport_core::{TransportCore, TransportType};
//use crate::module::joystick::JoyStick;
// use crate::module::imu::imu_type::MpuDevice;
use crate::module::ledmodule::Ledmodule;
// use crate::module::lidar::Lidar;
// use crate::module::range_finder::Rangefinder;
// use crate::module::remote_receiver::RemoteReceiverButton;
// use crate::module::servomodule::ServoModule;
use crate::module::stepper::{StepperMotor, StepperPinMode, StepperPins};
// use pinora_protocol::command::IncomingCommand;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type ModuleHandle<'a> = Box<dyn Module + 'a>;

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
    let p = Peripherals::take()?;

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let transport_core = TransportType::Serial;
    let tansport = TransportCore::new(transport_core, sys_loop.clone(), nvs.clone(), p.modem)?;
    let sync_sender = tansport.emitter.clone();

    configure_console_uart()?;
    print_esp_system_info(sync_sender.clone())?;
    let mut modules: HashMap<String, ModuleHandle<'_>> = HashMap::new();
    let mut last_yield_us = now_us();

  

    let i2c = I2cDriver::new(
        p.i2c0,
        p.pins.gpio21,
        p.pins.gpio22,
        &I2cConfig::new().baudrate(100.kHz().into()),
    )?;
    // MRC522 RST      -> GPIO 16

    //    LEFT                                      RIGHT
    //┌──────────────────────────────────────────────┐
    //│ SDA │ SCK │ MOSI │ MISO │ IRQ │ GND │ RST │ 3.3V │
    //└──────────────────────────────────────────────┘

    let shared_i2c = Rc::new(RefCell::new(i2c));
    // let shared_spi = Rc::new(RefCell::new(spi));

    let hardware = HardwareContext::new(p.ledc.timer0, shared_i2c.clone())?;
    let shared = Rc::new(RefCell::new(hardware));
    //let rangefinder_i2c = RangefinderI2c::new(RcDevice::new(shared.borrow().i2c_bus.clone()));

    // let lidar ={
    //     let hardware = shared.borrow();
    //     Lidar::new(
    //     hardware.servo_pwm.clone(),
    //     "lidar".to_string(),
    //     rangefinder_i2c,
    //     sync_sender.clone()
    // )?
    // };
    // let lidar_id = lidar.get_id().to_owned();
    // modules.insert(lidar_id, Box::new(lidar));

    // let stepperx = {
    //     StepperMotor::new(
    //         StepperPinMode::Manuel(StepperPins {
    //             in1: OutputPinCore::new(p.pins.gpio19)?,
    //             in2: OutputPinCore::new(p.pins.gpio18)?,
    //             in3: OutputPinCore::new(p.pins.gpio5)?,
    //             in4: OutputPinCore::new(p.pins.gpio17)?,
    //         }),
    //         "stepperx".to_string(),
    //         None,
    //         sync_sender.clone(),
    //     )
    //     .map_err(|err| anyhow::anyhow!("{err:?}"))?
    // };
    // let stepperx_id = stepperx.id().to_owned();
    // modules.insert(stepperx_id, Box::new(stepperx));

    // let joystick = {
        
    //     JoyStick::new(
    //         p.pins.gpio12,
    //         p.adc1,
    //         p.pins.gpio35,
    //         p.pins.gpio34,
    //         sync_sender.clone(),
    //     )
    //     .map_err(|err| anyhow::anyhow!("{err:?}"))?
    // };
    // let joystick_id = joystick.id().to_owned();
    // modules.insert(joystick_id, Box::new(joystick));

    // let remote_receiver = RemoteReceiverButton::new(
    //     InputPinCore::new(p.pins.gpio16, Pull::UpDown)
    //         .map_err(|err| anyhow::anyhow!("{err:?}"))?,
    //     "RemoteReceiver".to_string(),
    //     sync_sender.clone(),
    // )
    // .map_err(|err| anyhow::anyhow!("{err:?}"))?;
    // let remote_receiver_id = remote_receiver.id().to_owned();
    // modules.insert(remote_receiver_id, Box::new(remote_receiver));

    // let ranger ={
    //     Rangefinder::new(
    //         rangefinder_i2c,
    //         "ranger".to_string(),
    //         None,
    //         sync_sender.clone(),

    //     ).map_err(|err| anyhow::anyhow!("{err:?}"))?
    // };
    // let ranger_id = ranger.id().to_owned();
    // modules.insert(ranger_id, Box::new(ranger));

    // const MPU_ADDRESS: u8 = 0x68;
    // let imu_i2c = RcDevice::new(shared_i2c.clone());
    // let mut  test_imu = MpuDevice::new(imu_i2c, MPU_ADDRESS ,sync_sender.clone() , "MPu" , None ).map_err(|err| anyhow::anyhow!("{err:?}"))?;
    // modules.insert(test_imu.id().to_owned(), Box::new(test_imu));

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

    // let servo = {
    //     let hardware = shared.borrow();

    //     ServoModule::new(
    //         hardware.servo_pwm.clone(),
    //         "servo".to_string(),
    //         Channel::C0,
    //         ServoCapability{
    //             min_angle: 0,
    //             max_angle: 180,
    //             pulse_min: 500,
    //             pulse_max: 2500,
    //             max_pivot: 90,
    //             min_pivot: -90,
    //             offset: 90,
    //         },
    //         None,
    //         sync_sender.clone(),
    //     )?
    // };
    // let servo_id = servo.id().to_owned();
    // modules.insert(servo_id, Box::new(servo));

    let led1 = {
        let hardware = shared.borrow();

        Ledmodule::new(
            p.pins.gpio12,
            p.ledc.channel0,
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

        Ledmodule::new(
            p.pins.gpio14,
            p.ledc.channel1,
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

        Ledmodule::new(
            p.pins.gpio27,
            p.ledc.channel2,
            "led3".to_string(),
            &hardware.led_timer,
            None,
            sync_sender.clone(),
        )?
    };

    let led3_id = led3.id().to_owned();
    modules.insert(led3_id, Box::new(led3));

    for (_id, module) in modules.iter_mut() {
        module.register()?;
        module.first_emit()?;
    }

    // for module in modules.values() {
    //     module.register()?;
    //     module.first_emit()?;
    // }

    // let (command_sender, command_receiver) = mpsc::channel::<IncomingCommand>();
    // std::thread::spawn(move || {
    //     serial_command_reader(command_sender);
    // });
    let command_receiver = { tansport.handle_incoming() };

    loop {
        for module in modules.values_mut() {
            let _ = module.tick();
        }

        if let Ok(command) = command_receiver.try_recv() {
            if let Some(module) = modules.get_mut(&command.id) {
                module.handle_command(&command.command)?;
            } else if command.id == "SSI" {
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

