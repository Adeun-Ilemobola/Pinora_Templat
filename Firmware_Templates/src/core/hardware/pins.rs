//! Thin GPIO wrappers that remember their pin number.

use anyhow::Ok;
use esp_idf_svc::hal::gpio::*;

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
        Ok(self.driver.get_level().clone())
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
    pub fn high(&mut self) -> anyhow::Result<()> {
        self.driver.set_high()?;
        Ok(())
    }

    pub fn low(&mut self) -> anyhow::Result<()> {
        self.driver.set_low()?;
        Ok(())
    }

    pub fn toggle(&mut self) -> anyhow::Result<()> {
        self.driver.toggle()?;
        Ok(())
    }
}
