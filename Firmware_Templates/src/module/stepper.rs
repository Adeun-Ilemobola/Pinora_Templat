use std::{time::Instant};

use serde::{Deserialize, Serialize};

use crate::{
    core::{
        emitter::Emitter, hardware::{OutputPinCore, TimerState}, modulecore::{Module, ModuleCore}
    },
    protocol::{
        command::{ModuleCommand},
        global_definitions::{ModuleType},
        module_event::{ModuleEvent},
        registration::Registration,
    },
};

#[derive(Debug, Clone, Copy, PartialEq , Serialize, Deserialize)]
pub enum PivotPoint {
    Min,
    Max,
}
#[derive(Debug, Clone, Copy)]
pub struct PivotLimits {
    pub min: f32,
    pub max: f32,
}


#[derive(Debug, Clone, Copy, PartialEq ,Serialize, Deserialize,)]
pub enum StepperState {
    Idle,
    Moving,
    Homing {cycle :u32},
    Pivot {point:PivotPoint },
}

pub struct StepperPins<'d> {
    pub in1: OutputPinCore<'d>,
    pub in2: OutputPinCore<'d>,
    pub in3: OutputPinCore<'d>,
    pub in4: OutputPinCore<'d>,
}

const POSITIVE_SEQUENCE: [[u8; 4]; 8] = [
    [1, 0, 0, 0],
    [1, 1, 0, 0],
    [0, 1, 0, 0],
    [0, 1, 1, 0],
    [0, 0, 1, 0],
    [0, 0, 1, 1],
    [0, 0, 0, 1],
    [1, 0, 0, 1],
];

// const NEGATIVE_SEQUENCE: [[u8; 4]; 8] = [
//     [1, 0, 0, 1],
//     [0, 0, 0, 1],
//     [0, 0, 1, 1],
//     [0, 0, 1, 0],
//     [0, 1, 1, 0],
//     [0, 1, 0, 0],
//     [1, 1, 0, 0],
//     [1, 0, 0, 0],
// ];

pub struct StepperMotor<'d> {
    core: ModuleCore,
    pins: StepperPins<'d>,
    step_timer: TimerState,
    step: f32,
    target_step: f32,

    // origin: Option<f32>,
    mode: StepperState,
    test_time: Instant,
    pivot_point: PivotPoint,
    pivot_limits: PivotLimits,
}

impl<'d> StepperMotor<'d> {
    pub fn new(
        pins_bus: StepperPins<'d>,
        manuel_id: String,
        cluster_id: Option<String>,
        sender: Emitter,
    ) -> anyhow::Result<StepperMotor<'d>> {
        let motor = StepperMotor {
            core: ModuleCore::new(ModuleType::StepperMotor, &manuel_id, sender),
            pins: pins_bus,
            step_timer: TimerState::from_ms(1.0),
            target_step: 0.0,
            step: 0.0,
            mode: StepperState::Homing { cycle: 1 },
            test_time: Instant::now(),
            // origin: None,
            pivot_point: PivotPoint::Max,
            pivot_limits: PivotLimits::new(-90.0, 90.0),
        };
        if cluster_id.is_some() {}

        motor.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetAngle {
            id: motor.id().to_string(),
            angle: Self::step_to_angle(motor.step),
            step: motor.step,
        }));

        motor.registration(Registration {
            id: motor.id().to_string(),
            module_type: ModuleType::StepperMotor,
            lool_up_id: manuel_id.clone(),
            parent_id: cluster_id.clone().unwrap_or_default(),
        });

        Ok(motor)
    }
    pub fn angle_to_step(angle: f32) -> f32 {
        (angle * 4096.0 / 360.0).round()
    }
    pub fn step_to_angle(step: f32) -> f32 {
        step * 360.0 / 4096.0
    }
    pub fn target_step(&mut self, target: f32) {
        self.test_time = Instant::now();
        self.target_step = self.step + (target);
        self.step_timer.reset();
        self.mode = StepperState::Moving;
        self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetAngle {
            id: self.id().to_string(),
            angle: Self::step_to_angle(self.step),
            step: self.step,
        }));
        self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetMode {
            id: self.id().to_string(),
            mode: self.mode,
        }));
    }
    pub fn target_angle(&mut self, target: f32) {
        self.test_time = Instant::now();

        self.target_step = self.step + (Self::angle_to_step(target));

        self.step_timer.reset();
        self.mode = StepperState::Moving;
        self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetAngle {
            id: self.id().to_string(),
            angle: Self::step_to_angle(self.step),
            step: self.step,
        }));
        self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetMode {
            id: self.id().to_string(),
            mode: self.mode,
        }));
    }
    fn set_relative_angle_target(&mut self, angle: f32) {
        self.target_step = self.step + Self::angle_to_step(angle);

        self.step_timer.reset();
        self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetAngle {
            id: self.id().to_string(),
            angle: Self::step_to_angle(self.step),
            step: self.step,
        }));
        self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetMode {
            id: self.id().to_string(),
            mode: self.mode,
        }));
    }
    fn set_angle(&mut self, angle: f32) {
        self.target_step = Self::angle_to_step(angle);
        self.step_timer.reset();
        self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetAngle {
            id: self.id().to_string(),
            angle: Self::step_to_angle(self.step),
            step: self.step,
        }));
        self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetMode {
            id: self.id().to_string(),
            mode: self.mode,
        }));
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        match self.mode {
            StepperState::Idle => {}

            StepperState::Homing { mut cycle } => {
                if cycle > 6 {
                    self.mode = StepperState::Idle;
                    self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetMode {
                        id: self.id().to_string(),
                        mode: self.mode,
                    }));
                    return Ok(());
                }
                match self.pivot_point {
                    PivotPoint::Max => self.move_to_min_pivot()?,
                    PivotPoint::Min => self.move_to_max_pivot()?,
                }

                // if cycle % 2 == 1 {
                //     self.move_to_max_pivot()?;
                // } else {
                //     self.move_to_min_pivot()?;
                // }

                if self.step == self.target_step {
                    cycle += 1;
                    let elapsed = self.test_time.elapsed();
                    println!(
                        "Reached {:.2}° in {:?}",
                        Self::step_to_angle(self.step),
                        elapsed
                    );

                    match self.pivot_point {
                        PivotPoint::Max => self.pivot_point = PivotPoint::Min,
                        PivotPoint::Min => self.pivot_point = PivotPoint::Max,
                    }

                    self.emit(ModuleEvent::StepperMotor(
                        StepperMotorEvent::GetPivotPoint {
                            id: self.id().to_string(),
                            pivot_point: self.pivot_point.clone(),
                        },
                    ));

                    self.step_timer.reset();
                    self.mode = StepperState::Homing { cycle };
                }
            }

            StepperState::Moving => {
                self.go_to_position();
                if self.step == self.target_step {
                    self.mode = StepperState::Idle;
                    self.target_step = 0.0;
                    self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetMode {
                        id: self.id().to_string(),
                        mode: self.mode,
                    }));
                }
            }
            StepperState::Pivot { point } => match point {
                PivotPoint::Max => {
                    self.move_to_max_pivot()?;
                }
                PivotPoint::Min => {
                    self.move_to_min_pivot()?;
                }
            },
        }

        Ok(())
    }

    pub fn move_to_max_pivot(&mut self) -> anyhow::Result<()> {
        if self.target_step != Self::angle_to_step(self.pivot_limits.value(self.pivot_point)) {
            self.set_angle(self.pivot_limits.value(self.pivot_point));
        }
        self.go_to_position();
        Ok(())
    }
    pub fn move_to_min_pivot(&mut self) -> anyhow::Result<()> {
        if self.target_step != Self::angle_to_step(self.pivot_limits.value(self.pivot_point)) {
            // self.set_angle(self.pivot_limits.value(self.pivot_point));
            self.set_relative_angle_target(self.pivot_limits.value(self.pivot_point));
        }
        self.go_to_position();
        Ok(())
    }

    fn go_to_position(&mut self) {
        if self.step == self.target_step {
            return;
        }

        // Not yet time for the next electrical step.
        if !self.step_timer.ready() {
            return;
        }

        let direction = if self.target_step > self.step {
            1.0
        } else {
            -1.0
        };

        let next_step = self.step + direction;
        let sequence_index = next_step.rem_euclid(8.0) as usize;
        let sequence = POSITIVE_SEQUENCE[sequence_index];

        let _ = self.pins.in1.set_state(sequence[0] == 1);
        let _ = self.pins.in2.set_state(sequence[1] == 1);
        let _ = self.pins.in3.set_state(sequence[2] == 1);
        let _ = self.pins.in4.set_state(sequence[3] == 1);

        self.step = next_step;
    }
}

impl<'d> Module for StepperMotor<'d> {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::StepperMotor(payload) => match payload {
                StepperMotorCommandPayload::MoveToAngle { angle } => {
                    self.set_angle(*angle);
                    self.mode = StepperState::Moving;
                    self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetMode {
                        id: self.id().to_string(),
                        mode: self.mode,
                    }));
                }
                StepperMotorCommandPayload::SetPivotMax { pivot_max } => {
                    self.pivot_limits.update_max(*pivot_max);
                    self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetPivotMax {
                        id: self.id().to_string(),
                        pivot_max: *pivot_max,
                    }));
                }
                StepperMotorCommandPayload::SetPivotMin { pivot_min } => {
                    self.pivot_limits.update_min(*pivot_min);
                    self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetPivotMin {
                        id: self.id().to_string(),
                        pivot_min: *pivot_min,
                    }));
                }
                StepperMotorCommandPayload::MoveToOrigin => {
                    self.set_angle(0.0);
                    self.go_to_position();
                }
                StepperMotorCommandPayload::MoveToPivotMax => {
                    self.pivot_point = PivotPoint::Max;
                    self.move_to_max_pivot()?;
                }
                StepperMotorCommandPayload::MoveToPivotMin => {
                    self.pivot_point = PivotPoint::Min;
                    self.move_to_min_pivot()?;
                }
                StepperMotorCommandPayload::SetMode { mode } => {
                    self.mode = *mode;
                }
            },
            _ => {
                // Ignore commands intended for other module types.
            }
        }
        Ok(())
    }
}





impl PivotLimits {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn value(&self, point: PivotPoint) -> f32 {
        match point {
            PivotPoint::Min => self.min,
            PivotPoint::Max => self.max,
        }
    }

    pub fn opposite(&self, point: PivotPoint) -> PivotPoint {
        match point {
            PivotPoint::Min => PivotPoint::Max,
            PivotPoint::Max => PivotPoint::Min,
        }
    }
    pub fn update_max(&mut self , n:f32){ self.max = n}
    pub fn update_min(&mut self , n:f32){ self.min = n}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq,)]
#[serde(tag = "command")]
pub enum StepperMotorCommandPayload {
    SetPivotMin { pivot_min: f32 },
    SetPivotMax { pivot_max: f32 },
    MoveToOrigin,
    MoveToAngle { angle: f32 },
    MoveToPivotMin,
    MoveToPivotMax,
    SetMode { mode: StepperState },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, )]
#[serde(tag = "event_type")]
pub  enum  StepperMotorEvent {
     GetAngle { id: String, angle: f32 , step: f32},
     GetPivotMin { id: String, pivot_min: f32 },
     GetPivotMax { id: String, pivot_max: f32 },
     GetMode { id: String, mode: StepperState },
     GetOrigin { id: String, origin: Option<f32> },
     GetPivotPoint { id: String, pivot_point: PivotPoint },

}