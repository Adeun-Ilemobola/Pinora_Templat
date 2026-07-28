use std::time::Instant;

use crate::{
    core::{
        hardware::TimerState,
        modulecore::{emit, Module, ModuleCore},
    },
    protocol::{
        command::ModuleCommand,
        global_definitions::{ModuleType, StepperPins, StepperState},
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

const NEGATIVE_SEQUENCE: [[u8; 4]; 8] = [
    [1, 0, 0, 1],
    [0, 0, 0, 1],
    [0, 0, 1, 1],
    [0, 0, 1, 0],
    [0, 1, 1, 0],
    [0, 1, 0, 0],
    [1, 1, 0, 0],
    [1, 0, 0, 0],
];

pub struct StepperMotor<'d> {
    core: ModuleCore,
    pins: StepperPins<'d>,
    step_timer: TimerState,
    step: i32,
    target_step: i32,
    mode: StepperState,
    test_time:Instant
}

impl<'d> StepperMotor<'d> {
    pub fn new(
        pins_bus: StepperPins<'d>,
        manuel_id: String,
        cluster_id: Option<String>,
    ) -> anyhow::Result<StepperMotor<'d>> {
        let mut motor = StepperMotor {
            core: ModuleCore::new(ModuleType::StepperMotor, &manuel_id),
            pins: pins_bus,
            step_timer: TimerState::from_ms(1),
            target_step: 0,
            step: 0,
            mode: StepperState::Idle,
            test_time:Instant::now()
        };
        if cluster_id.is_some() {}

        let angle = motor.step as f32 * 360.0 / 4096.0;

        emit::event(ModuleEvent::StepperMotor(StepperMotorEvent::GetAngle {
            id: motor.id().clone(),
            angle,
        }));

        motor.move_to_target(4096);

        Ok(motor)
    }
    pub fn move_to_target(&mut self, target: i32) {
        self.test_time = Instant::now();
        self.target_step = target;
        self.step_timer.reset();
        self.mode = StepperState::Moving;
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        match self.mode {
            StepperState::Idle => {
                
            }

            StepperState::Homing => {
                self.move_to_target(0);
            }

            StepperState::Moving => {
                self.go_to_position();

                if self.step == self.target_step {
                    self.mode = StepperState::Idle;
                    let end  = self.test_time.elapsed();
                    println!("
                    \n
                    main sleep : 0 
                    [0 -> 360] : {:?}
                    \n
                    ",end)
                }
            }
        }

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

        let direction = if self.target_step > self.step { 1 } else { -1 };

        let next_step = self.step + direction;
        let sequence_index = next_step.rem_euclid(8) as usize;
        let sequence = POSITIVE_SEQUENCE[sequence_index];

        let _ = self.pins.in1.set_state(sequence[0] == 1);
        let _ = self.pins.in2.set_state(sequence[1] == 1);
        let _ = self.pins.in3.set_state(sequence[2] == 1);
        let _ = self.pins.in4.set_state(sequence[3] == 1);

        self.step = next_step;

        if self.step == self.target_step {
            let angle = self.step as f32 * 360.0 / 4096.0;

            emit::event(ModuleEvent::StepperMotor(StepperMotorEvent::GetAngle {
                id: self.id().clone(),
                angle,
            }));
        }
    }
}

impl<'d> Module for StepperMotor<'d> {
    fn id(&self) -> &String {
        &self.core.id
    }

    fn core(&self) -> &ModuleCore {
        &self.core
    }
    fn get_module_type(&self) -> &ModuleType {
        &self.core.module_type
    }
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()> {
        match command {
            _ => {
                // handle anything else
            }
        }
        Ok(())
    }
}
