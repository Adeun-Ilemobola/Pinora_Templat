use std::{sync::mpsc::SyncSender, time::Instant};

use crate::{
    core::{
        hardware::TimerState,
        modulecore::{emit, Module, ModuleCore},
    },
    protocol::{
        command::ModuleCommand,
        global_definitions::{ModuleType, PivotLimits, PivotPoint, StepperPins, StepperState},
        module_event::{ModuleEvent, StepperMotorEvent},
    },
};

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

    origin: Option<f32>,

    mode: StepperState,
    test_time: Instant,
    calibration_point: Option<usize>,
    calibration_angles: [f32; 7],

    trigger_angle: Option<f32>,
    angle_return: f32,
    current_trigger_count: u8,
    max_trigger_count: u8,
    pivot_point: PivotPoint,
    pivot_limits: PivotLimits,
}

impl<'d> StepperMotor<'d> {
    pub fn new(
        pins_bus: StepperPins<'d>,
        manuel_id: String,
        cluster_id: Option<String>,
        sender: SyncSender<ModuleEvent>,
    ) -> anyhow::Result<StepperMotor<'d>> {
        let mut motor = StepperMotor {
            core: ModuleCore::new(ModuleType::StepperMotor, &manuel_id, sender),
            pins: pins_bus,
            step_timer: TimerState::from_ms(0.5),
            target_step: 0.0,
            step: 0.0,
            mode: StepperState::Idle,
            test_time: Instant::now(),
            origin: None,
            trigger_angle: None,
            angle_return: 0.0,
            current_trigger_count: 1,
            max_trigger_count: 2,

            calibration_point: None,
            calibration_angles: [
                rand::random_range(0.0..=100.0),
                -rand::random_range(101.0..=200.0),
                rand::random_range(201.0..=300.0),
                rand::random_range(301.0..=400.0),
                -rand::random_range(401.0..=500.0),
                rand::random_range(501.0..=700.0),
                0.0,
            ],
            pivot_point: PivotPoint::Max,
            pivot_limits: PivotLimits::new(-90.0, 90.0),
        };
        if cluster_id.is_some() {}

        motor.emit(ModuleEvent::StepperMotor(StepperMotorEvent::GetAngle {
            id: motor.id().to_string(),
            angle: Self::step_to_angle(motor.step),
        }));

        motor.mode = StepperState::Homing;

        Ok(motor)
    }
    pub fn angle_to_step(angle: f32) -> f32 {
        (angle * 4096.0 / 360.0).round()
    }
    pub fn step_to_angle(step: f32) -> f32 {
        step * 360.0 / 4096.0
    }
    pub fn target_step(&mut self, target: f32) {
        println!(
            "current: {}, input: {}, new target: {}",
            self.step,
            target,
            self.step + target
        );
        self.test_time = Instant::now();
        self.target_step = self.step + (target);
        self.step_timer.reset();
        self.mode = StepperState::Moving;
    }
    pub fn target_angle(&mut self, target: f32) {
        println!(
            "current: {}, input: {}, new target: {}",
            self.step,
            target,
            self.step + target
        );
        self.test_time = Instant::now();

        self.target_step = self.step + (Self::angle_to_step(target));

        self.step_timer.reset();
        self.mode = StepperState::Moving;
    }
    fn set_relative_angle_target(&mut self, angle: f32) {
        self.target_step = self.step + Self::angle_to_step(angle);

        self.step_timer.reset();
    }
    fn set_angle(&mut self, angle: f32) {
        self.target_step = Self::angle_to_step(angle);

        self.step_timer.reset();
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        match self.mode {
            StepperState::Idle => {}

            StepperState::Homing => {
                if self.step == self.target_step {
                    let elapsed = self.test_time.elapsed();

                    println!(
                        "Reached {:.2}° in {:?}",
                        Self::step_to_angle(self.step),
                        elapsed
                    );
                    self.current_trigger_count += 1;

                    if self.current_trigger_count >= self.max_trigger_count {
                        self.mode = StepperState::Pivot;
                        return Ok(());
                    }

                    if self.current_trigger_count % 2 == 1 {
                        self.set_relative_angle_target(90.0);
                    } else {
                        self.set_relative_angle_target(-180.0);
                    }
                    self.step_timer.reset();
                }

                self.go_to_position();
            }

            StepperState::Moving => {
                self.go_to_position();

                if self.step == self.target_step {}
            }
            StepperState::Pivot => match self.pivot_point {
                PivotPoint::Max => {
                    if  self.current_trigger_count == 5{
                         self.mode = StepperState::Idle;
                        return Ok(());

                    }
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
        if self.target_step != self.pivot_limits.value(self.pivot_point) {
            self.set_angle(self.pivot_limits.value(self.pivot_point));
        }

        if self.step == self.target_step {
            self.pivot_point = PivotPoint::Min;
            return Ok(());
        }
        println!("Current angle : {} ", Self::step_to_angle(self.step));

        self.go_to_position();
        Ok(())
    }
    pub fn move_to_min_pivot(&mut self) -> anyhow::Result<()> {
        if self.target_step != self.pivot_limits.value(self.pivot_point) {
            self.set_angle(self.pivot_limits.value(self.pivot_point));
        }
        if self.step == self.target_step {
            self.pivot_point = PivotPoint::Max;
            self.current_trigger_count += 1;
            return Ok(());
        }
        println!("Current angle : {} ", Self::step_to_angle(self.step));

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

    fn start_next_calibration_move(&mut self) {
        self.calibration_point = Some(0);
        self.target_angle(self.calibration_angles[0]);
    }

    fn advance_calibration(&mut self) {
        let Some(current_point) = self.calibration_point else {
            self.mode = StepperState::Idle;
            return;
        };

        let next_point = current_point + 1;
        if next_point < self.calibration_angles.len() {
            self.calibration_point = Some(next_point);
            self.target_angle(self.calibration_angles[next_point]);
        } else {
            self.finish_calibration();
        }
    }

    fn finish_calibration(&mut self) {
        self.calibration_point = None;
        self.mode = StepperState::Idle;
        log::info!("Stepper calibration finished");
    }
}

impl<'d> Module for StepperMotor<'d> {
    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            ModuleCommand::StepperMotor(payload) => match payload {
                crate::protocol::command::StepperMotorCommandPayload::SetPositionAngle {
                    angle,
                } => {
                    self.calibration_point = None;
                    self.target_angle(*angle as f32);
                }
                crate::protocol::command::StepperMotorCommandPayload::SetPositionSequences => {
                    self.mode = StepperState::Homing;
                }
            },
            _ => {
                // Ignore commands intended for other module types.
            }
        }
        Ok(())
    }
}
