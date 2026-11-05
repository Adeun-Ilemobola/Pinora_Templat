//! Pure stepper motion logic (PIN-TPL-004, Phase 1D).
//!
//! Owns the mode, position and target in half-steps, the limits, and the homing and
//! pivot sequencing. `StepperMotor` only checks the step timer and drives the coils.
//! No ESP-IDF types, so this is tested on the host (`tests/firmware-host`).
//!
//! Safety rules enforced here:
//! * The motor starts `Idle` and moves only after a command.
//! * Every target is finite and within the travel limit; pivot limits stay within
//!   travel with `min <= max`. Violations are rejected and nothing moves.
//! * Pivot and homing targets are **absolute**. The pre-1D code added them to the
//!   current position, which could rotate without bound.
//! * When the motor becomes `Idle` the coils are released, unless holding is enabled.

use std::fmt;

use pinora_protocol::module::stepper::{
    PivotPoint, StepperMotorCommandPayload, StepperMotorEvent, StepperStateType,
};

/// Half-steps per output-shaft revolution (28BYJ-48 with its gearbox).
pub const STEPS_PER_REV: f32 = 4096.0;
/// Default travel limit either side of the origin, in degrees.
pub const DEFAULT_TRAVEL_DEG: f32 = 180.0;
/// Default pivot limits, in degrees.
pub const DEFAULT_PIVOT_MIN_DEG: f32 = -90.0;
pub const DEFAULT_PIVOT_MAX_DEG: f32 = 90.0;
/// Number of pivot-to-pivot sweeps `Homing` performs before returning to `Idle`.
pub const HOMING_CYCLES: u32 = 3;

pub fn angle_to_step(angle: f32) -> f32 {
    (angle * STEPS_PER_REV / 360.0).round()
}

pub fn step_to_angle(step: f32) -> f32 {
    step * 360.0 / STEPS_PER_REV
}

/// Index into the 8-entry half-step coil sequence for a position.
pub fn sequence_index(step: f32) -> usize {
    step.rem_euclid(8.0) as usize
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Idle,
    Moving,
    Homing { cycle: u32, toward: PivotPoint },
    Pivot { point: PivotPoint },
}

impl Mode {
    pub fn kind(&self) -> StepperStateType {
        match self {
            Mode::Idle => StepperStateType::Idle,
            Mode::Moving => StepperStateType::Moving,
            Mode::Homing { .. } => StepperStateType::Homing,
            Mode::Pivot { .. } => StepperStateType::Pivot,
        }
    }
}

/// A command that was rejected; nothing moved.
#[derive(Debug, Clone, PartialEq)]
pub enum MotionError {
    /// An angle outside `-travel..=travel` (or not finite).
    OutsideTravel { angle: f32, travel: f32 },
    /// Pivot limits that would leave `min > max`.
    InvertedPivotLimits { min: f32, max: f32 },
}

impl fmt::Display for MotionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MotionError::OutsideTravel { angle, travel } => {
                write!(
                    f,
                    "stepper angle {angle} is outside the travel limit ±{travel}°"
                )
            }
            MotionError::InvertedPivotLimits { min, max } => {
                write!(
                    f,
                    "stepper pivot limits would be inverted (min {min} > max {max})"
                )
            }
        }
    }
}

impl std::error::Error for MotionError {}

/// What the hardware module must do after a command or tick.
#[derive(Debug, Default, PartialEq)]
pub struct Output {
    pub events: Vec<StepperMotorEvent>,
    /// De-energize all coils.
    pub release_coils: bool,
}

#[derive(Debug, Clone)]
pub struct StepperMotion {
    step: f32,
    target: f32,
    mode: Mode,
    /// The pivot point the motor is resting on; `None` once it has moved off it.
    pivot_point: Option<PivotPoint>,
    pivot_min: f32,
    pivot_max: f32,
    travel: f32,
    hold_when_idle: bool,
}

impl Default for StepperMotion {
    fn default() -> Self {
        Self::new()
    }
}

impl StepperMotion {
    /// Idle at the origin with the default limits. Nothing moves until a command.
    pub fn new() -> Self {
        Self {
            step: 0.0,
            target: 0.0,
            mode: Mode::Idle,
            pivot_point: None,
            pivot_min: DEFAULT_PIVOT_MIN_DEG,
            pivot_max: DEFAULT_PIVOT_MAX_DEG,
            travel: DEFAULT_TRAVEL_DEG,
            hold_when_idle: false,
        }
    }

    /// Keep the coils energized when idle (holding torque). Off by default.
    pub fn set_hold_when_idle(&mut self, hold: bool) {
        self.hold_when_idle = hold;
    }

    pub fn step(&self) -> f32 {
        self.step
    }
    pub fn target(&self) -> f32 {
        self.target
    }
    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn pivot_point(&self) -> Option<PivotPoint> {
        self.pivot_point
    }
    pub fn pivot_limits(&self) -> (f32, f32) {
        (self.pivot_min, self.pivot_max)
    }

    fn check_travel(&self, angle: f32) -> Result<f32, MotionError> {
        if angle.is_finite() && angle.abs() <= self.travel {
            Ok(angle)
        } else {
            Err(MotionError::OutsideTravel {
                angle,
                travel: self.travel,
            })
        }
    }

    fn pivot_value(&self, point: PivotPoint) -> f32 {
        match point {
            PivotPoint::Min => self.pivot_min,
            PivotPoint::Max => self.pivot_max,
        }
    }

    fn mode_event(&self) -> StepperMotorEvent {
        StepperMotorEvent::GetMode {
            mode: self.mode.kind(),
        }
    }

    fn angle_event(&self) -> StepperMotorEvent {
        StepperMotorEvent::GetAngle {
            angle: step_to_angle(self.step),
            step: self.step,
        }
    }

    /// The pivot limit further from where the motor is now. Homing starts there, so its
    /// first sweep always moves, wherever the motor was left.
    fn farther_pivot(&self) -> PivotPoint {
        let to_min = (self.step - angle_to_step(self.pivot_min)).abs();
        let to_max = (self.step - angle_to_step(self.pivot_max)).abs();
        if to_min >= to_max {
            PivotPoint::Min
        } else {
            PivotPoint::Max
        }
    }

    /// Enters `Idle`: stop at the current position, report it, and release the coils
    /// unless holding is enabled.
    fn become_idle(&mut self, output: &mut Output) {
        self.mode = Mode::Idle;
        self.target = self.step;
        output.release_coils = !self.hold_when_idle;
        output.events.push(self.angle_event());
        output.events.push(self.mode_event());
    }

    /// Applies a command. On `Err` the state is unchanged and nothing moves.
    pub fn handle(&mut self, command: &StepperMotorCommandPayload) -> Result<Output, MotionError> {
        let mut output = Output::default();
        match command {
            StepperMotorCommandPayload::MoveToAngle { angle } => {
                self.target = angle_to_step(self.check_travel(*angle)?);
                self.mode = Mode::Moving;
                output.events.push(self.mode_event());
            }
            StepperMotorCommandPayload::MoveToOrigin {} => {
                self.target = 0.0;
                self.mode = Mode::Moving;
                output.events.push(self.mode_event());
            }
            StepperMotorCommandPayload::SetPivotMax { pivot_max } => {
                let max = self.check_travel(*pivot_max)?;
                if max < self.pivot_min {
                    return Err(MotionError::InvertedPivotLimits {
                        min: self.pivot_min,
                        max,
                    });
                }
                self.pivot_max = max;
                output.events.push(StepperMotorEvent::GetPivotMax {
                    pivot_max: self.pivot_max,
                });
            }
            StepperMotorCommandPayload::SetPivotMin { pivot_min } => {
                let min = self.check_travel(*pivot_min)?;
                if min > self.pivot_max {
                    return Err(MotionError::InvertedPivotLimits {
                        min,
                        max: self.pivot_max,
                    });
                }
                self.pivot_min = min;
                output.events.push(StepperMotorEvent::GetPivotMin {
                    pivot_min: self.pivot_min,
                });
            }
            StepperMotorCommandPayload::MoveToPivotMax {} => {
                self.mode = Mode::Pivot {
                    point: PivotPoint::Max,
                };
                output.events.push(self.mode_event());
            }
            StepperMotorCommandPayload::MoveToPivotMin {} => {
                self.mode = Mode::Pivot {
                    point: PivotPoint::Min,
                };
                output.events.push(self.mode_event());
            }
            StepperMotorCommandPayload::SetMode { mode } => match mode {
                StepperStateType::Idle => self.become_idle(&mut output),
                StepperStateType::Homing => {
                    self.mode = Mode::Homing {
                        cycle: 1,
                        toward: self.farther_pivot(),
                    };
                    output.events.push(self.mode_event());
                }
                StepperStateType::Pivot => {
                    self.mode = Mode::Pivot {
                        point: PivotPoint::Max,
                    };
                    output.events.push(self.mode_event());
                }
                StepperStateType::Moving => {
                    self.mode = Mode::Moving;
                    output.events.push(self.mode_event());
                }
            },
        }
        Ok(output)
    }

    /// Advances one tick. `try_step(sequence_index)` is called at most once, only
    /// when a step is wanted; it returns `Ok(true)` if it drove the coils (the step
    /// timer was due), `Ok(false)` if it is not yet time.
    pub fn tick<E>(
        &mut self,
        mut try_step: impl FnMut(usize) -> Result<bool, E>,
    ) -> Result<Output, E> {
        let mut output = Output::default();
        match self.mode {
            Mode::Idle => {}
            Mode::Moving => {
                self.step_toward_target(&mut try_step)?;
                if self.step == self.target {
                    self.become_idle(&mut output);
                }
            }
            Mode::Homing { cycle, toward } => {
                if cycle > HOMING_CYCLES {
                    self.become_idle(&mut output);
                    return Ok(output);
                }
                self.target = angle_to_step(self.pivot_value(toward));
                self.step_toward_target(&mut try_step)?;
                if self.step == self.target {
                    self.pivot_point = Some(toward);
                    log::info!("Stepper homing reached {:.2}°", step_to_angle(self.step));
                    output
                        .events
                        .push(StepperMotorEvent::GetPivotPoint { pivot_point: toward });
                    output.events.push(self.angle_event());
                    output.events.push(self.mode_event());
                    self.mode = Mode::Homing {
                        cycle: cycle + 1,
                        toward: match toward {
                            PivotPoint::Max => PivotPoint::Min,
                            PivotPoint::Min => PivotPoint::Max,
                        },
                    };
                }
            }
            Mode::Pivot { point } => {
                self.target = angle_to_step(self.pivot_value(point));
                self.step_toward_target(&mut try_step)?;
                if self.step == self.target {
                    self.pivot_point = Some(point);
                    output
                        .events
                        .push(StepperMotorEvent::GetPivotPoint { pivot_point: point });
                    self.become_idle(&mut output);
                }
            }
        }
        Ok(output)
    }

    fn step_toward_target<E>(
        &mut self,
        try_step: &mut impl FnMut(usize) -> Result<bool, E>,
    ) -> Result<(), E> {
        if self.step == self.target {
            return Ok(());
        }
        let direction = if self.target > self.step { 1.0 } else { -1.0 };
        let next = self.step + direction;
        if try_step(sequence_index(next))? {
            self.step = next;
            self.pivot_point = None;
        }
        Ok(())
    }
}
