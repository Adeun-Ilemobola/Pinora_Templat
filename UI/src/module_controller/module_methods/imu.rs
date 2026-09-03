use pinora_protocol::{
    Axes as ProtocolAxes, ImuEvent, MpuDeviceMode as ProtocolMpuDeviceMode,
    RawAxes as ProtocolRawAxes,
};

use crate::{Axes, ImuState, MpuDeviceMode, RawAxes};

impl From<ProtocolAxes> for Axes {
    fn from(axes: ProtocolAxes) -> Self {
        Self {
            x: axes.x,
            y: axes.y,
            z: axes.z,
        }
    }
}

impl From<Axes> for ProtocolAxes {
    fn from(axes: Axes) -> Self {
        Self {
            x: axes.x,
            y: axes.y,
            z: axes.z,
        }
    }
}

impl From<ProtocolRawAxes> for RawAxes {
    fn from(axes: ProtocolRawAxes) -> Self {
        Self {
            x: i32::from(axes.x),
            y: i32::from(axes.y),
            z: i32::from(axes.z),
        }
    }
}

impl TryFrom<RawAxes> for ProtocolRawAxes {
    type Error = std::num::TryFromIntError;

    fn try_from(axes: RawAxes) -> Result<Self, Self::Error> {
        Ok(Self {
            x: i16::try_from(axes.x)?,
            y: i16::try_from(axes.y)?,
            z: i16::try_from(axes.z)?,
        })
    }
}

impl From<ProtocolMpuDeviceMode> for MpuDeviceMode {
    fn from(mode: ProtocolMpuDeviceMode) -> Self {
        match mode {
            ProtocolMpuDeviceMode::Collecting => Self::Collecting,
            ProtocolMpuDeviceMode::Idle => Self::Idle,
            ProtocolMpuDeviceMode::Off => Self::Off,
        }
    }
}

impl From<MpuDeviceMode> for ProtocolMpuDeviceMode {
    fn from(mode: MpuDeviceMode) -> Self {
        match mode {
            MpuDeviceMode::Collecting => Self::Collecting,
            MpuDeviceMode::Idle => Self::Idle,
            MpuDeviceMode::Off => Self::Off,
        }
    }
}

impl ImuState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, event: ImuEvent) {
        match event {
            ImuEvent::Gyro { id, raw_axes, axes } => {
                self.has_id = true;
                self.id = id.into();
                self.has_gyro_raw_axes = true;
                self.gyro_raw_axes = raw_axes.into();
                self.has_gyro_axes = true;
                self.gyro_axes = axes.into();
            }
            ImuEvent::Accel { id, raw_axes, axes } => {
                self.has_id = true;
                self.id = id.into();
                self.has_accel_raw_axes = true;
                self.accel_raw_axes = raw_axes.into();
                self.has_accel_axes = true;
                self.accel_axes = axes.into();
            }
            ImuEvent::Mode { mode } => {
                self.has_mode = true;
                self.mode = mode.into();
            }
        }
    }
}
