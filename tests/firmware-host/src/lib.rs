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
        #[path = "../../../../../Firmware_Templates/src/core/transport/emitter.rs"]
        pub mod emitter;

        #[path = "../../../../../Firmware_Templates/src/core/transport/discovery_cache.rs"]
        pub mod discovery_cache;
    }
}

pub mod utilities {
    #[path = "../../../../Firmware_Templates/src/utilities/math.rs"]
    pub mod math;
}
