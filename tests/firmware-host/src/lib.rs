//! Firmware sources compiled for the host.
//!
//! The module tree mirrors `Firmware_Templates/src` so the included files resolve
//! their `crate::…` paths unchanged. Only files with no ESP-IDF dependency may be
//! included here; if a firmware edit adds one, this crate stops compiling, which is
//! the intended alarm. Paths are relative to each inline module's directory.

pub mod core {
    #[path = "../../../../Firmware_Templates/src/core/module.rs"]
    pub mod module;

    pub mod transport {
        #[path = "../../../../../Firmware_Templates/src/core/transport/auth_gate.rs"]
        pub mod auth_gate;

        #[path = "../../../../../Firmware_Templates/src/core/transport/emitter.rs"]
        pub mod emitter;

        #[path = "../../../../../Firmware_Templates/src/core/transport/discovery_cache.rs"]
        pub mod discovery_cache;

        #[path = "../../../../../Firmware_Templates/src/core/transport/drop_stats.rs"]
        pub mod drop_stats;

        #[path = "../../../../../Firmware_Templates/src/core/transport/pairing_window.rs"]
        pub mod pairing_window;
    }
}

pub mod modules {
    pub mod actuators {
        #[path = "../../../../../Firmware_Templates/src/modules/actuators/led_mapping.rs"]
        pub mod led_mapping;

        #[path = "../../../../../Firmware_Templates/src/modules/actuators/stepper_motion.rs"]
        pub mod stepper_motion;
    }

    pub mod composite {
        #[path = "../../../../../Firmware_Templates/src/modules/composite/lidar_scan.rs"]
        pub mod lidar_scan;
    }
}

pub mod runtime {
    #[path = "../../../../Firmware_Templates/src/runtime/dispatch.rs"]
    pub mod dispatch;

    #[path = "../../../../Firmware_Templates/src/runtime/tick_errors.rs"]
    pub mod tick_errors;
}

pub mod utilities {
    #[path = "../../../../Firmware_Templates/src/utilities/math.rs"]
    pub mod math;

    #[path = "../../../../Firmware_Templates/src/utilities/rate_limit.rs"]
    pub mod rate_limit;
}
