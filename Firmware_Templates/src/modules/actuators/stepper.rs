//! Unipolar stepper (28BYJ-48 style, 4096 half-steps per revolution), driven directly
//! on four GPIOs or through a 74HC595 shift register.
//!
//! Motion decisions (modes, limits, homing and pivot targets, when to release the
//! coils) live in the pure [`super::stepper_motion`] core, which is host-tested. This
//! module only checks the step timer, drives the coils and emits events.
//!
//! Safety (PIN-TPL-004, Phase 1D): the motor starts `Idle` and moves only on command.
//! Out-of-range targets are rejected (reported by the dispatcher as a `SysLog`). Coils
//! are de-energized when idle unless [`StepperMotor::set_hold_when_idle`] is enabled.
//! Coil-write errors are returned instead of panicking. Not instantiated in the
//! current image.

use crate::core::{
    hardware::{OutputPinCore, TimerState},
    module::{Module, ModuleCore, ModuleError},
};

use crate::core::transport::emitter::{EmitterError, TransportEmitter};
use crate::modules::actuators::stepper_motion::{step_to_angle, Output, StepperMotion};

use pinora_protocol::{
    command::ModuleCommand, global_definitions::ModuleType, module_event::ModuleEvent,
};

pub use pinora_protocol::module::stepper::{
    PivotPoint, StepperMotorCommandPayload, StepperMotorEvent, StepperStateType,
};

pub struct StepperPins<'d> {
    pub in1: OutputPinCore<'d>,
    pub in2: OutputPinCore<'d>,
    pub in3: OutputPinCore<'d>,
    pub in4: OutputPinCore<'d>,
}
pub struct StepperPinAuto<'d> {
    pub data: OutputPinCore<'d>,
    pub clock: OutputPinCore<'d>,
    pub latch: OutputPinCore<'d>,
}

pub enum StepperPinMode<'d> {
    Manuel(StepperPins<'d>),
    Auto(StepperPinAuto<'d>),
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

const SEQUENCE_74HC595: [u8; 8] = [
    0b0000_0001, // IN1
    0b0000_0011, // IN1 + IN2
    0b0000_0010, // IN2
    0b0000_0110, // IN2 + IN3
    0b0000_0100, // IN3
    0b0000_1100, // IN3 + IN4
    0b0000_1000, // IN4
    0b0000_1001, // IN4 + IN1
];
pub struct StepperMotor<'d> {
    core: ModuleCore,
    step_timer: TimerState,
    motion: StepperMotion,
    motion_mode: StepperPinMode<'d>,
}

impl<'d> StepperMotor<'d> {
    /// Creates the stepper `Idle` at the origin. No coil is driven here.
    pub fn new(
        motion_mode: StepperPinMode<'d>,
        lookup_name: String,
        cluster_id: Option<String>,
        sender: TransportEmitter,
    ) -> anyhow::Result<StepperMotor<'d>> {
        let motor = StepperMotor {
            core: ModuleCore::new(ModuleType::StepperMotor, &lookup_name, cluster_id, sender),
            step_timer: TimerState::from_ms(1.25),
            motion: StepperMotion::new(),
            motion_mode,
        };
        Ok(motor)
    }

    /// Keep the coils energized when idle (holding torque). Off by default.
    pub fn set_hold_when_idle(&mut self, hold: bool) {
        self.motion.set_hold_when_idle(hold);
    }

    fn tick_inner(&mut self) -> anyhow::Result<()> {
        let timer = &mut self.step_timer;
        let pins = &mut self.motion_mode;
        let output = self.motion.tick(|sequence_index| {
            // Not yet time for the next electrical step.
            if !timer.ready() {
                return Ok(false);
            }
            write_coils(pins, Some(sequence_index))?;
            timer.reset();
            Ok::<bool, anyhow::Error>(true)
        })?;
        self.apply(output)
    }

    fn apply(&mut self, output: Output) -> anyhow::Result<()> {
        if output.release_coils {
            write_coils(&mut self.motion_mode, None)?;
        }
        for event in output.events {
            self.emit(ModuleEvent::StepperMotor(event));
        }
        Ok(())
    }
}

/// Drives one half-step pattern, or de-energizes every coil when `sequence_index` is
/// `None`.
fn write_coils(pins: &mut StepperPinMode<'_>, sequence_index: Option<usize>) -> anyhow::Result<()> {
    match pins {
        StepperPinMode::Auto(p) => {
            let sequence = sequence_index.map_or(0, |index| SEQUENCE_74HC595[index]);
            // Keep latch LOW while loading bits.
            p.latch.low()?;

            // Send 8 bits, MSB first.
            for bit in (0..8).rev() {
                let is_high = (sequence & (1 << bit)) != 0;

                if is_high {
                    p.data.high()?;
                } else {
                    p.data.low()?;
                }

                // Rising edge tells 74HC595:
                // "take whatever is currently on DATA"
                p.clock.high()?;
                p.clock.low()?;
            }
            // Transfer the completed byte to Q0-Q7.
            p.latch.high()?;
            p.latch.low()?;
        }
        StepperPinMode::Manuel(p) => {
            let sequence = sequence_index.map_or([0; 4], |index| POSITIVE_SEQUENCE[index]);
            p.in1.set_state(sequence[0] == 1)?;
            p.in2.set_state(sequence[1] == 1)?;
            p.in3.set_state(sequence[2] == 1)?;
            p.in4.set_state(sequence[3] == 1)?;
        }
    };
    Ok(())
}

impl<'d> Module for StepperMotor<'d> {
    fn register(&self) -> Result<(), EmitterError> {
        self.emit_registration()?;
        self.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetAngle {
            angle: step_to_angle(self.motion.step()),
            step: self.motion.step(),
        }));
        Ok(())
    }

    fn tick(&mut self) -> Result<(), ModuleError> {
        self.tick_inner().map_err(|_| ModuleError::OperationFailed)
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }
    /// Commands outside the limits return an error (reported by the dispatcher) and
    /// do not move the motor.
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::StepperMotor(payload) => {
                let output = self.motion.handle(payload)?;
                self.apply(output)?;
            }
            _ => {
                // Ignore commands intended for other module types.
            }
        }
        Ok(())
    }
}
