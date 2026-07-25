use anyhow::Ok;
use embedded_hal_bus::i2c::RcDevice;
use embedded_hal_compat::Reverse;
pub use esp_idf_svc::hal::delay::FreeRtos;
pub use esp_idf_svc::hal::gpio::*;
pub use esp_idf_svc::hal::i2c;
pub use esp_idf_svc::hal::i2c::{I2c, I2cConfig, I2cDriver};
pub use esp_idf_svc::hal::ledc;
use esp_idf_svc::hal::ledc::config::TimerConfig;
use esp_idf_svc::hal::ledc::Resolution;
pub use esp_idf_svc::hal::peripherals::Peripherals;
pub use esp_idf_svc::hal::uart::UartDriver;
pub use esp_idf_svc::hal::units::*;
pub use esp_idf_svc::partition::*;
use esp_idf_svc::sys;
use pinora_shared::protocol::registration::SystemInfo;
use pwm_pca9685::{Address, Pca9685};
use std::{ffi::CStr, ptr};

use crate::core::modulecore::emit;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub struct OutputPinCore<'d> {
    pin_number: u8,
    driver: PinDriver<'d, Output>,
}
pub struct InputPinCore<'d> {
    pin_number: u8,
    driver: PinDriver<'d, Input>,
}

impl<'d> InputPinCore<'d> {
    pub fn new<T>(pin: T, pull_mode: Pull) -> anyhow::Result<Self>
    where
        T: InputPin + 'd,
    {
        let pin_number = pin.pin() as u8;
        let driver = PinDriver::input(pin, pull_mode)?;

        Ok(Self { pin_number, driver })
    }
    pub fn pin_number(&self) -> u8 {
        self.pin_number
    }

    pub fn high(&self) -> anyhow::Result<bool> {
        Ok(self.driver.is_high())
    }
    pub fn low(&self) -> anyhow::Result<bool> {
        Ok(self.driver.is_low())
    }
    pub fn now(&self) -> anyhow::Result<Level> {
        Ok(self.driver.get_level())
    }
}

impl<'d> OutputPinCore<'d> {
    pub fn new<T>(pin: T) -> anyhow::Result<Self>
    where
        T: OutputPin + 'd,
    {
        let pin_number = pin.pin() as u8;
        let driver = PinDriver::output(pin)?;
        Ok(Self { pin_number, driver })
    }

    pub fn pin_number(&self) -> u8 {
        self.pin_number
    }

    pub fn set_state(&mut self, state: bool) -> anyhow::Result<()> {
        if state {
            self.driver.set_high()?;
        } else {
            self.driver.set_low()?;
        }

        Ok(())
    }

    pub fn toggle(&mut self) -> anyhow::Result<()> {
        self.driver.toggle()?;
        Ok(())
    }
}

pub fn sleep_time(ms: u32) {
    FreeRtos::delay_ms(ms);
}
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

pub type I2cBus<'d> = Rc<RefCell<I2cDriver<'d>>>;
pub type SharedI2cDevice<'d> = RcDevice<I2cDriver<'d>>;

pub type RangefinderI2c<'d> = Reverse<RcDevice<I2cDriver<'d>>>;

pub type SharedPwm<'d> = Rc<RefCell<Pca9685<SharedI2cDevice<'d>>>>;

pub type LedTimer<'d> = ledc::LedcTimerDriver<'d, ledc::LowSpeed>;

pub struct HardwareContext<'d> {
    pub servo_pwm: SharedPwm<'d>,
    pub led_timer: LedTimer<'d>,
    pub i2c_bus: I2cBus<'d>,
}

impl<'d> HardwareContext<'d> {
    pub fn new<TIMER>(timer: TIMER, i2c_bus: I2cBus<'d>) -> anyhow::Result<HardwareContext<'d>>
    where
        TIMER: ledc::LedcTimer<SpeedMode = ledc::LowSpeed> + 'd,
    {
        Ok(Self {
            servo_pwm: Self::create_shared_pwm(i2c_bus.clone())?,
            led_timer: Self::create_led_timer(timer)?,
            i2c_bus,
        })
    }

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

pub fn print_esp_system_info() {
    unsafe {
        // -------------------------
        // RAM / heap information
        // -------------------------
        let memory_caps = sys::MALLOC_CAP_8BIT as u32;

        let total_heap = sys::heap_caps_get_total_size(memory_caps);

        let free_heap = sys::heap_caps_get_free_size(memory_caps);

        let minimum_free_heap = sys::heap_caps_get_minimum_free_size(memory_caps);

        let largest_free_block = sys::heap_caps_get_largest_free_block(memory_caps);

        // -------------------------
        // Flash size
        // -------------------------
        let mut flash_size = 0_u32;

        let flash_result = sys::esp_flash_get_size(ptr::null_mut(), &mut flash_size);

        // -------------------------
        // Maximum firmware/app slot
        // -------------------------
        let running_partition = sys::esp_ota_get_running_partition();

        let app_partition_size = if running_partition.is_null() {
            0
        } else {
            (*running_partition).size
        };

        // -------------------------
        // ESP-IDF version
        // -------------------------
        let idf_version = CStr::from_ptr(sys::esp_get_idf_version()).to_string_lossy();
        let flash = if flash_result == sys::ESP_OK {
            format_bytes(flash_size as usize)
        } else {
            "Failed to read".to_string()
        };

        emit::system_info(SystemInfo {
            esp_idf_version: idf_version.to_string(),
            total_heap: format_bytes(total_heap),
            current_free_heap: format_bytes(free_heap),
            lowest_free_heap: format_bytes(minimum_free_heap),
            largest_allocation: format_bytes(largest_free_block),
            maximum_app_slot: format_bytes(app_partition_size as usize),
            flash,
        });
    }
}

fn format_bytes(bytes: usize) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;

    if bytes as f64 >= MIB {
        format!("{:.2} MiB ({bytes} bytes)", bytes as f64 / MIB)
    } else {
        format!("{:.2} KiB ({bytes} bytes)", bytes as f64 / KIB)
    }
}
