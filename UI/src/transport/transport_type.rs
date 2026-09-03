use pinora_protocol::ModuleType;

use crate::{
    UiModuleType,
    slint_generatedAppWindow::{ConnectionTypeS, TransportTypeS},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConnectionType {
    Disconnected,
    Connecting,
    Connected,
    Error,
}
impl ConnectionType {
    pub fn to_slint(&self) -> ConnectionTypeS {
        match self {
            Self::Disconnected => ConnectionTypeS::Disconnected,
            Self::Connecting => ConnectionTypeS::Connecting,
            Self::Connected => ConnectionTypeS::Connected,
            Self::Error => ConnectionTypeS::Error,
        }
    }
}

pub fn to_slint_model(values: Vec<String>) -> slint::ModelRc<slint::SharedString> {
    let model_values = values
        .into_iter()
        .map(|value| value.into())
        .collect::<Vec<_>>();

    slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(model_values)))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransportType {
    None,
    Serial,
    Wifi,
    Bluetooth,
}
impl TransportType {
    pub const fn to_array() -> [Self; 4] {
        [Self::None, Self::Serial, Self::Wifi, Self::Bluetooth]
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Serial => "Serial",
            Self::Wifi => "Wifi",
            Self::Bluetooth => "Bluetooth",
        }
    }
    pub fn to_slint_model() -> slint::ModelRc<slint::SharedString> {
        let values = Self::to_array()
            .iter()
            .filter(|value| **value != Self::None)
            .map(|value| value.as_str().into())
            .collect::<Vec<_>>();

        slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(values)))
    }
    pub fn format(&self) -> TransportTypeS {
        match self {
            Self::None => TransportTypeS::Serial,
            Self::Serial => TransportTypeS::Serial,
            Self::Wifi => TransportTypeS::Wifi,
            Self::Bluetooth => TransportTypeS::Bluetooth,
        }
    }
    pub fn to_self(s: TransportTypeS) -> Self {
        match s {
            TransportTypeS::Serial => Self::Serial,
            TransportTypeS::Wifi => Self::Wifi,
            TransportTypeS::Bluetooth => Self::Bluetooth,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BaudRate {
    B9600,
    B19200,
    B38400,
    B57600,
    B115200,
    B230400,
    B460800,
    B921600,
}

impl BaudRate {
    pub const fn to_array() -> [Self; 8] {
        [
            Self::B9600,
            Self::B19200,
            Self::B38400,
            Self::B57600,
            Self::B115200,
            Self::B230400,
            Self::B460800,
            Self::B921600,
        ]
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            Self::B9600 => 9600,
            Self::B19200 => 19200,
            Self::B38400 => 38400,
            Self::B57600 => 57600,
            Self::B115200 => 115200,
            Self::B230400 => 230400,
            Self::B460800 => 460800,
            Self::B921600 => 921600,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::B9600 => "9600",
            Self::B19200 => "19200",
            Self::B38400 => "38400",
            Self::B57600 => "57600",
            Self::B115200 => "115200",
            Self::B230400 => "230400",
            Self::B460800 => "460800",
            Self::B921600 => "921600",
        }
    }
    pub fn from_label(s: &str) -> Self {
        match s {
            "9600" => Self::B9600,
            "19200" => Self::B19200,
            "38400" => Self::B38400,
            "57600" => Self::B57600,
            "115200" => Self::B115200,
            "230400" => Self::B230400,
            "460800" => Self::B460800,
            "921600" => Self::B921600,
            _ => Self::B115200,
        }
    }
    pub fn to_slint_model() -> slint::ModelRc<slint::SharedString> {
        let values = Self::to_array()
            .iter()
            .map(|value| value.as_str().into())
            .collect::<Vec<_>>();

        slint::ModelRc::from(std::rc::Rc::new(slint::VecModel::from(values)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransportError {
    ConnectionFailed {
        message: String,
        raw_error: Option<String>,
    },
    Timeout,
    Unknown {
        message: String,
        raw_error: Option<String>,
    },
}

pub struct ConnectionState {
    pub connection_type: ConnectionType,
    pub transport_type: Option<TransportType>,
    pub error: Option<TransportError>,
}

impl From<ModuleType> for UiModuleType {
    fn from(ty: ModuleType) -> Self {
        match ty {
            ModuleType::Servo => UiModuleType::Servo,
            ModuleType::Led => UiModuleType::Led,
            ModuleType::Imu => UiModuleType::Imu,
            ModuleType::LedCluster => UiModuleType::LedCluster,
            ModuleType::Button => UiModuleType::Button,
            ModuleType::Lidar => UiModuleType::Lidar,
            ModuleType::Rangefinder => UiModuleType::Rangefinder,
            ModuleType::SysLog => UiModuleType::SysLog,
            ModuleType::JoyStick => UiModuleType::JoyStick,
            ModuleType::StepperMotor => UiModuleType::StepperMotor,
            ModuleType::Rfid => UiModuleType::Rfid,
            ModuleType::RemoteReceiver => UiModuleType::RemoteReceiver,
        }
    }
}

impl From<UiModuleType> for ModuleType {
    fn from(ty: UiModuleType) -> Self {
        match ty {
            UiModuleType::Servo => ModuleType::Servo,
            UiModuleType::Led => ModuleType::Led,
            UiModuleType::Imu => ModuleType::Imu,
            UiModuleType::LedCluster => ModuleType::LedCluster,
            UiModuleType::Button => ModuleType::Button,
            UiModuleType::Lidar => ModuleType::Lidar,
            UiModuleType::Rangefinder => ModuleType::Rangefinder,
            UiModuleType::SysLog => ModuleType::SysLog,
            UiModuleType::JoyStick => ModuleType::JoyStick,
            UiModuleType::StepperMotor => ModuleType::StepperMotor,
            UiModuleType::Rfid => ModuleType::Rfid,
            UiModuleType::RemoteReceiver => ModuleType::RemoteReceiver,
        }
    }
}
