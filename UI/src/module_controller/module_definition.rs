use pinora_protocol::{
    Axes, ButtonEvent, ImuEvent, LedEvent, LidarEvent, LogPriority, MddeRfid, ModuleEvent,
    MpuDeviceMode, PivotPoint, Point, RangPoint, RangefinderDistanceMode, RangefinderEvent,
    RawAxes, RemoteButton, RemoteButtonEvent, RfidEvent, ScanState, ServoEvent, StepperMotorEvent,
    StepperStateType, SysLogEvent, WriteState,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleState {
    Servo(ServoState),
    Led(LedState),
    Lidar(LidarState),
    Rangefinder(RangefinderState),
    Button(ButtonState),
    SysLog(SysLogState),
    RemoteReceiver(RemoteReceiverState),
    StepperMotor(StepperMotorState),
    Imu(ImuState),
    Rfid(RfidState),
}

impl ModuleState {
    pub fn new(event: ModuleEvent) -> Self {
        match event {
            ModuleEvent::Led(event) => {
                let mut state = LedState::new();
                state.update(event);
                Self::Led(state)
            }
            ModuleEvent::Button(event) => {
                let mut state = ButtonState::new();
                state.update(event);
                Self::Button(state)
            }
            ModuleEvent::SysLog(event) => {
                let mut state = SysLogState::new();
                state.update(event);
                Self::SysLog(state)
            }
            ModuleEvent::RemoteReceiver(event) => {
                let mut state = RemoteReceiverState::new();
                state.update(event);
                Self::RemoteReceiver(state)
            }
            ModuleEvent::StepperMotor(event) => {
                let mut state = StepperMotorState::new();
                state.update(event);
                Self::StepperMotor(state)
            }
            ModuleEvent::Imu(event) => {
                let mut state = ImuState::new();
                state.update(event);
                Self::Imu(state)
            }
            ModuleEvent::Rfid(event) => {
                let mut state = RfidState::new();
                state.update(event);
                Self::Rfid(state)
            }
        }
    }

    pub fn update(&mut self, event: ModuleEvent) {
        match (self, event) {
            (Self::Led(state), ModuleEvent::Led(event)) => state.update(event),
            (Self::Button(state), ModuleEvent::Button(event)) => state.update(event),
            (Self::SysLog(state), ModuleEvent::SysLog(event)) => state.update(event),
            (Self::RemoteReceiver(state), ModuleEvent::RemoteReceiver(event)) => {
                state.update(event)
            }
            (Self::StepperMotor(state), ModuleEvent::StepperMotor(event)) => state.update(event),
            (Self::Imu(state), ModuleEvent::Imu(event)) => state.update(event),
            (Self::Rfid(state), ModuleEvent::Rfid(event)) => state.update(event),
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LedState {
    pub id: String,
    pub brightness: u32,
}

impl LedState {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            brightness: 0,
        }
    }

    pub fn update(&mut self, event: LedEvent) {
        match event {
            LedEvent::Brightness { id, level } => {
                self.id = id;
                self.brightness = level;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServoState {
    pub id: String,
    pub angle: Option<i32>,
    pub min_pivot: Option<i32>,
    pub max_pivot: Option<i32>,
    pub offset: Option<i32>,
}

impl ServoState {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            angle: None,
            min_pivot: None,
            max_pivot: None,
            offset: None,
        }
    }

    pub fn update(&mut self, event: ServoEvent) {
        match event {
            ServoEvent::GetAngle { id, angle } => {
                self.id = id;
                self.angle = Some(angle);
            }
            ServoEvent::GetMinPivot { id, min_pivot } => {
                self.id = id;
                self.min_pivot = Some(min_pivot);
            }
            ServoEvent::GetMaxPivot { id, max_pivot } => {
                self.id = id;
                self.max_pivot = Some(max_pivot);
            }
            ServoEvent::GetOffset { id, angle } => {
                self.id = id;
                self.offset = Some(angle);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LidarState {
    pub id: String,
    pub roi_min: Option<Point>,
    pub roi_max: Option<Point>,
    pub max_chunk: Option<i32>,
    pub current_chunk: Option<i32>,
    pub map: Option<Vec<RangPoint>>,
    pub target: Option<Point>,
    pub scan_state: Option<ScanState>,
    pub scan_time: Option<f32>,
}

impl LidarState {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            roi_min: None,
            roi_max: None,
            max_chunk: None,
            current_chunk: None,
            map: None,
            target: None,
            scan_state: None,
            scan_time: None,
        }
    }

    pub fn update(&mut self, event: LidarEvent) {
        match event {
            LidarEvent::Roi { id, min, max } => {
                self.id = id;
                self.roi_min = Some(min);
                self.roi_max = Some(max);
            }
            LidarEvent::PointMap {
                id,
                max_chunk,
                curr_chunk,
                map,
            } => {
                self.id = id;
                self.max_chunk = Some(max_chunk);
                self.current_chunk = Some(curr_chunk);
                self.map = Some(map);
            }
            LidarEvent::Target { id, point } => {
                self.id = id;
                self.target = Some(point);
            }
            LidarEvent::ScanState {
                id,
                state,
                scan_time,
            } => {
                self.id = id;
                self.scan_state = Some(state);
                self.scan_time = Some(scan_time);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangefinderState {
    pub id: String,
    pub millimeters: Option<u16>,
    pub is_ranging: Option<bool>,
    pub timing_budget_milliseconds: Option<u16>,
    pub distance_mode: Option<RangefinderDistanceMode>,
    pub invalid_measurement_status: Option<String>,
}

impl RangefinderState {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            millimeters: None,
            is_ranging: None,
            timing_budget_milliseconds: None,
            distance_mode: None,
            invalid_measurement_status: None,
        }
    }

    pub fn update(&mut self, event: RangefinderEvent) {
        match event {
            RangefinderEvent::Range { id, millimeters } => {
                self.id = id;
                self.millimeters = Some(millimeters);
            }
            RangefinderEvent::RangingState { id, is_ranging } => {
                self.id = id;
                self.is_ranging = Some(is_ranging);
            }
            RangefinderEvent::TimingBudget { id, milliseconds } => {
                self.id = id;
                self.timing_budget_milliseconds = Some(milliseconds);
            }
            RangefinderEvent::DistanceMode { id, mode } => {
                self.id = id;
                self.distance_mode = Some(mode);
            }
            RangefinderEvent::InvalidMeasurement { id, status } => {
                self.id = id;
                self.invalid_measurement_status = Some(status);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonState {
    pub id: String,
}

impl ButtonState {
    pub fn new() -> Self {
        Self { id: String::new() }
    }

    pub fn update(&mut self, event: ButtonEvent) {
        match event {
            ButtonEvent::Ckick { id } => self.id = id,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SysLogState {
    pub text: String,
    pub raw_err: Option<String>,
    pub priority: LogPriority,
}

impl SysLogState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            raw_err: None,
            priority: LogPriority::Low,
        }
    }

    pub fn update(&mut self, event: SysLogEvent) {
        self.text = event.text;
        self.raw_err = event.raw_err;
        self.priority = event.priority;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoteReceiverState {
    pub id: String,
    pub key: RemoteButton,
}

impl RemoteReceiverState {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            key: RemoteButton::None,
        }
    }

    pub fn update(&mut self, event: RemoteButtonEvent) {
        match event {
            RemoteButtonEvent::Click { id, key } => {
                self.id = id;
                self.key = key;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepperMotorState {
    pub id: String,
    pub angle: Option<f32>,
    pub step: Option<f32>,
    pub pivot_min: Option<f32>,
    pub pivot_max: Option<f32>,
    pub mode: Option<StepperStateType>,
    pub origin: Option<f32>,
    pub pivot_point: Option<PivotPoint>,
}

impl StepperMotorState {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            angle: None,
            step: None,
            pivot_min: None,
            pivot_max: None,
            mode: None,
            origin: None,
            pivot_point: None,
        }
    }

    pub fn update(&mut self, event: StepperMotorEvent) {
        match event {
            StepperMotorEvent::GetAngle { id, angle, step } => {
                self.id = id;
                self.angle = Some(angle);
                self.step = Some(step);
            }
            StepperMotorEvent::GetPivotMin { id, pivot_min } => {
                self.id = id;
                self.pivot_min = Some(pivot_min);
            }
            StepperMotorEvent::GetPivotMax { id, pivot_max } => {
                self.id = id;
                self.pivot_max = Some(pivot_max);
            }
            StepperMotorEvent::GetMode { id, mode } => {
                self.id = id;
                self.mode = Some(mode);
            }
            StepperMotorEvent::GetOrigin { id, origin } => {
                self.id = id;
                self.origin = origin;
            }
            StepperMotorEvent::GetPivotPoint { id, pivot_point } => {
                self.id = id;
                self.pivot_point = Some(pivot_point);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImuState {
    pub id: Option<String>,
    pub gyro_raw_axes: Option<RawAxes>,
    pub gyro_axes: Option<Axes>,
    pub accel_raw_axes: Option<RawAxes>,
    pub accel_axes: Option<Axes>,
    pub mode: Option<MpuDeviceMode>,
}

impl ImuState {
    pub fn new() -> Self {
        Self {
            id: None,
            gyro_raw_axes: None,
            gyro_axes: None,
            accel_raw_axes: None,
            accel_axes: None,
            mode: None,
        }
    }

    pub fn update(&mut self, event: ImuEvent) {
        match event {
            ImuEvent::Gyro { id, raw_axes, axes } => {
                self.id = Some(id);
                self.gyro_raw_axes = Some(raw_axes);
                self.gyro_axes = Some(axes);
            }
            ImuEvent::Accel { id, raw_axes, axes } => {
                self.id = Some(id);
                self.accel_raw_axes = Some(raw_axes);
                self.accel_axes = Some(axes);
            }
            ImuEvent::Mode { mode } => self.mode = Some(mode),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RfidState {
    pub id: String,
    pub card_uid: Option<String>,
    pub card_data: Option<String>,
    pub mode: Option<MddeRfid>,
    pub write_state: Option<WriteState>,
    pub info: Option<String>,
}

impl RfidState {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            card_uid: None,
            card_data: None,
            mode: None,
            write_state: None,
            info: None,
        }
    }

    pub fn update(&mut self, event: RfidEvent) {
        match event {
            RfidEvent::GetCard {
                id,
                card_uid,
                card_data,
            } => {
                self.id = id;
                self.card_uid = Some(card_uid);
                self.card_data = Some(card_data);
            }
            RfidEvent::GetMode { id, mode } => {
                self.id = id;
                self.mode = Some(mode);
            }
            RfidEvent::GetWriteState { id, state, info } => {
                self.id = id;
                self.write_state = Some(state);
                self.info = Some(info);
            }
        }
    }
}
