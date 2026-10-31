//! Hardware modules, grouped by role. Each implements `core::module::Module`.
//!
//! Only `actuators::led` is instantiated by the current image (see `board`).

pub mod actuators;
pub mod composite;
pub mod sensors;
