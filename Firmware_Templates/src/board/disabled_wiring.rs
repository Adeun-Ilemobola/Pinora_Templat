//! Wiring for module families that are compiled but **not** instantiated in the
//! current image. It is kept verbatim from the previous `main.rs`, as comments, for
//! reference. Nothing in this file runs.
//!
//! # Read before enabling anything here
//!
//! * **Motion.** Since Phase 1D nothing moves during construction or registration,
//!   stepper targets are bounded, and LiDAR scans terminate (PIN-TPL-004/-005). The
//!   servo and LiDAR sweeps still exist as explicit `run_self_test()` calls, which
//!   move hardware. Verify on a supervised bench before enabling (1D result D-V5).
//! * **Pin conflicts with the active LEDs** (GPIO 12/14/27): the joystick button uses
//!   GPIO12, and the RFID RGB LED uses GPIO 12/14/27 (PIN-TPL-018).
//! * **Bluetooth has no application-level authorization** (PIN-TPL-003). Pairing is
//!   limited to 120 s after boot (Phase 1C), but any device paired in that window
//!   could drive these actuators until Phase 2B adds authentication.
//! * The servo and LiDAR blocks need `HardwareContext::servo_pwm`, which is currently
//!   commented out in `core/hardware/context.rs`. The RFID block needs an SPI driver
//!   that is not created anywhere.
//!
//! The snippets were written inside `main()` and refer to its locals (`p`, `shared`,
//! `shared_i2c`, `modules`, `sync_sender`).

// MRC522 RST      -> GPIO 16

//    LEFT                                      RIGHT
//┌──────────────────────────────────────────────┐
//│ SDA │ SCK │ MOSI │ MISO │ IRQ │ GND │ RST │ 3.3V │
//└──────────────────────────────────────────────┘

// let shared_spi = Rc::new(RefCell::new(spi));

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

// Previously used imports, for when these blocks are re-enabled:
// use crate::modules::sensors::joystick::JoyStick;
// use crate::modules::sensors::imu::types::MpuDevice;
// use crate::modules::composite::lidar::Lidar;
// use crate::modules::sensors::rangefinder::Rangefinder;
// use crate::modules::sensors::remote_receiver::RemoteReceiverButton;
// use crate::modules::actuators::servo::ServoModule;
// use crate::modules::actuators::stepper::{StepperMotor, StepperPinMode, StepperPins};
// use embedded_hal_bus::i2c::RcDevice;
// use pinora_protocol::ServoCapability;
// use pwm_pca9685::Channel;
