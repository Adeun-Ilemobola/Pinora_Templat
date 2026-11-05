//! Modules that drive outputs. Servo and stepper are not instantiated in the current
//! image; since Phase 1D neither moves during construction (PIN-TPL-004).

pub mod led;
pub mod led_mapping;
pub mod servo;
pub mod servo_position;
pub mod stepper;
pub mod stepper_motion;
