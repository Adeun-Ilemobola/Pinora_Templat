//! Firmware sources compiled for the host.
//!
//! Each hardware-free firmware file is included once, as a flat (hidden) module whose
//! `#[path]` is relative to this `src/` directory. The public tree below then mirrors
//! `Firmware_Templates/src` with re-exports, so the included files resolve their
//! `crate::…` paths unchanged. Only files with no ESP-IDF dependency may be included;
//! if a firmware edit adds one, this crate stops compiling, which is the intended alarm.
//!
//! Why flat: a `#[path]` inside nested inline modules is resolved from directories
//! named after those modules (`src/core/transport/…`). Windows normalizes `..` in such
//! paths textually, but Linux resolves them through the filesystem, where those
//! directories do not exist, so the crate did not build on the Linux CI runner.

#[path = "../../../Firmware_Templates/src/board/pin_map.rs"]
#[doc(hidden)]
pub mod board_pin_map;

#[path = "../../../Firmware_Templates/src/core/module.rs"]
#[doc(hidden)]
pub mod core_module;
#[path = "../../../Firmware_Templates/src/core/transport/auth_gate.rs"]
#[doc(hidden)]
pub mod transport_auth_gate;
#[path = "../../../Firmware_Templates/src/core/transport/discovery_cache.rs"]
#[doc(hidden)]
pub mod transport_discovery_cache;
#[path = "../../../Firmware_Templates/src/core/transport/drop_stats.rs"]
#[doc(hidden)]
pub mod transport_drop_stats;
#[path = "../../../Firmware_Templates/src/core/transport/emitter.rs"]
#[doc(hidden)]
pub mod transport_emitter;
#[path = "../../../Firmware_Templates/src/core/transport/pairing_window.rs"]
#[doc(hidden)]
pub mod transport_pairing_window;

#[path = "../../../Firmware_Templates/src/modules/actuators/led_mapping.rs"]
#[doc(hidden)]
pub mod actuators_led_mapping;
#[path = "../../../Firmware_Templates/src/modules/actuators/stepper_motion.rs"]
#[doc(hidden)]
pub mod actuators_stepper_motion;
#[path = "../../../Firmware_Templates/src/modules/composite/lidar_scan.rs"]
#[doc(hidden)]
pub mod composite_lidar_scan;
#[path = "../../../Firmware_Templates/src/modules/sensors/nec.rs"]
#[doc(hidden)]
pub mod sensors_nec;
#[path = "../../../Firmware_Templates/src/modules/sensors/rfid_record.rs"]
#[doc(hidden)]
pub mod sensors_rfid_record;

#[path = "../../../Firmware_Templates/src/runtime/dispatch.rs"]
#[doc(hidden)]
pub mod runtime_dispatch;
#[path = "../../../Firmware_Templates/src/runtime/schedule.rs"]
#[doc(hidden)]
pub mod runtime_schedule;
#[path = "../../../Firmware_Templates/src/runtime/tick_errors.rs"]
#[doc(hidden)]
pub mod runtime_tick_errors;

#[path = "../../../Firmware_Templates/src/utilities/math.rs"]
#[doc(hidden)]
pub mod utilities_math;
#[path = "../../../Firmware_Templates/src/utilities/rate_limit.rs"]
#[doc(hidden)]
pub mod utilities_rate_limit;

pub mod board {
    pub use crate::board_pin_map as pin_map;
}

pub mod core {
    pub use crate::core_module as module;

    pub mod transport {
        pub use crate::transport_auth_gate as auth_gate;
        pub use crate::transport_discovery_cache as discovery_cache;
        pub use crate::transport_drop_stats as drop_stats;
        pub use crate::transport_emitter as emitter;
        pub use crate::transport_pairing_window as pairing_window;
    }
}

pub mod modules {
    pub mod actuators {
        pub use crate::actuators_led_mapping as led_mapping;
        pub use crate::actuators_stepper_motion as stepper_motion;
    }

    pub mod composite {
        pub use crate::composite_lidar_scan as lidar_scan;
    }

    pub mod sensors {
        pub use crate::sensors_nec as nec;
        pub use crate::sensors_rfid_record as rfid_record;
    }
}

pub mod runtime {
    pub use crate::runtime_dispatch as dispatch;
    pub use crate::runtime_schedule as schedule;
    pub use crate::runtime_tick_errors as tick_errors;
}

pub mod utilities {
    pub use crate::utilities_math as math;
    pub use crate::utilities_rate_limit as rate_limit;
}
