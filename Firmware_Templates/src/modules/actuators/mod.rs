//! Modules that drive outputs. Servo and stepper are not instantiated in the current
//! image and move hardware during construction (PIN-TPL-004).

pub mod led;
pub mod led_mapping;
pub mod servo;
pub mod stepper;
