use std::{rc::Rc, sync::mpsc::SyncSender};

use crate::core::transport::transport_emiter::{EmitterError, TransportEmiter};
use esp_idf_svc::hal::adc::{
    attenuation,
    oneshot::{config::AdcChannelConfig, AdcChannelDriver, AdcDriver},
    Adc, AdcChannel, AdcUnit,
};
use esp_idf_svc::hal::gpio::ADCPin;

use crate::core::hardware::InputPin;
use crate::core::modulecore::{Module, ModuleCore, ModuleError};
use crate::module::buttonmodule::Buttonmodule;
use crate::utilities::math::range_i16;
use pinora_protocol::{
    command::ModuleCommand, global_definitions::ModuleType, registration::ProtocolMessage,
};

pub struct JoyStick<'d, U, X, Y>
where
    U: AdcUnit,
    X: AdcChannel<AdcUnit = U>,
    Y: AdcChannel<AdcUnit = U>,
{
    core: ModuleCore,
    button: Buttonmodule<'d>,
    x_channel: AdcChannelDriver<'d, X, Rc<AdcDriver<'d, U>>>,
    y_channel: AdcChannelDriver<'d, Y, Rc<AdcDriver<'d, U>>>,
    x_value_raw: u16,
    y_value_raw: u16,
    x_value: i16,
    y_value: i16,
}

const X_CENTRE: u16 = 2610;
const Y_CENTRE: u16 = 2750;
const DEADZONE: u16 = 200;
const CHANGE_THRESHOLD: i16 = 2;

impl<'d, U, X, Y> JoyStick<'d, U, X, Y>
where
    U: AdcUnit,
    X: AdcChannel<AdcUnit = U>,
    Y: AdcChannel<AdcUnit = U>,
{
    pub fn new<MB, ADC, AX, AY>(
        mb: MB,
        adc: ADC,
        ax: AX,
        ay: AY,
        sender: TransportEmiter,
    ) -> anyhow::Result<Self>
    where
        MB: InputPin + 'd,
        ADC: Adc<AdcUnit = U> + 'd,
        AX: ADCPin<AdcChannel = X> + 'd,
        AY: ADCPin<AdcChannel = Y> + 'd,
    {
        let adc = Rc::new(AdcDriver::new(adc)?);
        let config = AdcChannelConfig {
            attenuation: attenuation::DB_12,
            ..Default::default()
        };

        Ok(Self {
            core: ModuleCore::new(ModuleType::JoyStick, "JoyStick", None, sender.clone()),
            button: Buttonmodule::new(mb, "mb".to_string(), sender.clone())?,
            x_channel: AdcChannelDriver::new(adc.clone(), ax, &config)?,
            y_channel: AdcChannelDriver::new(adc, ay, &config)?,
            x_value_raw: 0,
            y_value_raw: 0,
            x_value: 0,
            y_value: 0,
        })
    }

    fn axis_value(raw: u16 , centre: u16) -> i16 {
        if raw.abs_diff(centre) <= DEADZONE {
            0
        } else {
            range_i16(raw as i16, 0, 4095, -30, 30)
        }
    }

    fn read_x(&mut self) -> anyhow::Result<i16> {
        self.x_value_raw = self.x_channel.read_raw()?;
        Ok(Self::axis_value(self.x_value_raw, X_CENTRE))
    }

    fn read_y(&mut self) -> anyhow::Result<i16> {
        self.y_value_raw = self.y_channel.read_raw()?;
        Ok(Self::axis_value(self.y_value_raw, Y_CENTRE))
    }

    pub fn values(&self) -> (i16, i16) {
        (self.x_value, self.y_value)
    }

    fn tick_inner(&mut self) -> anyhow::Result<()> {
        let x = self.read_x()?;
        let y = self.read_y()?;
        let x_changed = (x - self.x_value).abs() >= CHANGE_THRESHOLD;
        let y_changed = (y - self.y_value).abs() >= CHANGE_THRESHOLD;

        if x_changed || y_changed {
            self.x_value = x;
            self.y_value = y;

            log::info!(
                "Joystick: x={} y={} (raw x={} y={})",
                self.x_value,
                self.y_value,
                self.x_value_raw,
                self.y_value_raw,
            );
        }

        if self.button.poll()? {
            log::info!("Joystick button clicked");
        }

        Ok(())
    }
}

impl<'d, U, X, Y> Module for JoyStick<'d, U, X, Y>
where
    U: AdcUnit,
    X: AdcChannel<AdcUnit = U>,
    Y: AdcChannel<AdcUnit = U>,
{
    fn register(&self) -> Result<(), EmitterError> {
        self.emit_registration()?;
        self.button.register()?;
        Ok(())
    }

    fn tick(&mut self) -> Result<(), ModuleError> {
        self.tick_inner().map_err(|_| ModuleError::OperationFailed)
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }

    fn handle_command(&mut self, _command: &ModuleCommand) -> anyhow::Result<()> {
        Ok(())
    }
}
