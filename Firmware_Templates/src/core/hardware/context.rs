//! Shared peripherals that several modules use at once: the LEDC timer and the I²C
//! bus (plus the PCA9685 PWM expander once servos are re-enabled).

use anyhow::Ok;
use embedded_hal_bus::i2c::RcDevice;
use embedded_hal_compat::Reverse;
use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::hal::ledc;
use esp_idf_svc::hal::ledc::config::TimerConfig;
use esp_idf_svc::hal::ledc::Resolution;
use esp_idf_svc::hal::units::*;
use pwm_pca9685::{Address, Pca9685};
use std::cell::RefCell;
use std::rc::Rc;

/// The I²C0 bus, shared between devices via [`RcDevice`]. Single-threaded on purpose.
pub type I2cBus<'d> = Rc<RefCell<I2cDriver<'d>>>;
pub type SharedI2cDevice<'d> = RcDevice<I2cDriver<'d>>;

/// The VL53L1X driver expects embedded-hal 0.2 traits, hence the `Reverse` adapter.
pub type RangefinderI2c<'d> = Reverse<RcDevice<I2cDriver<'d>>>;

pub type SharedPwm<'d> = Rc<RefCell<Pca9685<SharedI2cDevice<'d>>>>;

pub type LedTimer<'d> = ledc::LedcTimerDriver<'d, ledc::LowSpeed>;

pub struct HardwareContext<'d> {
    // pub servo_pwm: SharedPwm<'d>,
    pub led_timer: LedTimer<'d>,
    pub i2c_bus: I2cBus<'d>,
}

impl<'d> HardwareContext<'d> {
    pub fn new<TIMER>(timer: TIMER, i2c_bus: I2cBus<'d>) -> anyhow::Result<HardwareContext<'d>>
    where
        TIMER: ledc::LedcTimer<SpeedMode = ledc::LowSpeed> + 'd,
    {
        Ok(Self {
            //servo_pwm: Self::create_shared_pwm(i2c_bus.clone())?,
            led_timer: Self::create_led_timer(timer)?,
            i2c_bus,
        })
    }

    /// PCA9685 at its default address; prescale 100 gives about 60 Hz, which
    /// `utilities::math::pulse_us_to_tick` assumes.
    pub fn create_shared_pwm(i2c_bus: I2cBus<'d>) -> anyhow::Result<SharedPwm<'d>> {
        let i2c_device = RcDevice::new(i2c_bus);

        let mut pwm = Pca9685::new(i2c_device, Address::default())
            .map_err(|e| anyhow::anyhow!("PCA9685 init: {:?}", e))?;

        pwm.set_prescale(100)
            .map_err(|e| anyhow::anyhow!("set_prescale: {:?}", e))?;

        pwm.enable()
            .map_err(|e| anyhow::anyhow!("enable: {:?}", e))?;

        Ok(Rc::new(RefCell::new(pwm)))
    }

    /// One low-speed LEDC timer shared by all LEDs: 5 kHz, 13-bit (max duty 8191).
    pub fn create_led_timer<T>(timer: T) -> anyhow::Result<LedTimer<'d>>
    where
        T: ledc::LedcTimer<SpeedMode = ledc::LowSpeed> + 'd,
    {
        let timer_config = TimerConfig::new()
            .frequency(5_u32.kHz().into())
            .resolution(Resolution::Bits13);

        Ok(ledc::LedcTimerDriver::new(timer, &timer_config)?)
    }
}
