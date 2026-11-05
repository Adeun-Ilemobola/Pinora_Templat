//! A servo's position and limits, in one place (found 2026-11-05).
//!
//! `angle` is the only stored position; the pivot is always worked out from it, so the
//! two cannot disagree. A move writes the pulse first and saves the angle only if the
//! write succeeded, so a failed write leaves the position where the servo really is.
//! Hobby servos have no feedback: "position" means the last pulse that was written.
//!
//! No ESP-IDF types, so this is tested on the host (`tests/firmware-host`).

use pinora_protocol::module::servomodule::ServoCapability;

use crate::utilities::math::range_i32;

#[derive(Debug, Clone)]
pub struct ServoPosition {
    config: ServoCapability,
    offset: i32,
    angle: i32,
    min_pivot: i32,
    max_pivot: i32,
    commanded: bool,
}

impl ServoPosition {
    pub fn new(config: &ServoCapability) -> Self {
        let offset = config.offset.clamp(config.min_angle, config.max_angle);
        Self {
            config: config.clone(),
            offset,
            angle: offset,
            min_pivot: config.min_pivot,
            max_pivot: config.max_pivot,
            commanded: false,
        }
    }

    /// Clamps `pivot` to the limits, calls `write` with the pulse in microseconds, and
    /// saves the new angle only if `write` succeeded.
    pub fn move_to<E>(
        &mut self,
        pivot: i32,
        write: impl FnOnce(i32) -> Result<(), E>,
    ) -> Result<(), E> {
        let pivot = pivot.clamp(self.min_pivot, self.max_pivot);
        let angle = (self.offset + pivot).clamp(self.config.min_angle, self.config.max_angle);
        write(self.pulse_for(angle))?;
        self.angle = angle;
        self.commanded = true;
        Ok(())
    }

    /// Changes the offset and drives the servo to it (pivot 0), saving both only if
    /// `write` succeeded.
    pub fn move_to_new_offset<E>(
        &mut self,
        offset: i32,
        write: impl FnOnce(i32) -> Result<(), E>,
    ) -> Result<(), E> {
        let offset = offset.clamp(self.config.min_angle, self.config.max_angle);
        write(self.pulse_for(offset))?;
        self.offset = offset;
        self.angle = offset;
        self.commanded = true;
        Ok(())
    }

    fn pulse_for(&self, angle: i32) -> i32 {
        range_i32(
            angle,
            self.config.min_angle,
            self.config.max_angle,
            self.config.pulse_min,
            self.config.pulse_max,
        )
    }

    /// Where the servo is, relative to its offset.
    pub fn pivot(&self) -> i32 {
        self.angle - self.offset
    }

    pub fn angle(&self) -> i32 {
        self.angle
    }

    pub fn offset(&self) -> i32 {
        self.offset
    }

    /// False until the first successful write: until then the real position is unknown.
    pub fn commanded(&self) -> bool {
        self.commanded
    }

    pub fn pivot_limits(&self) -> (i32, i32) {
        (self.min_pivot, self.max_pivot)
    }

    /// Stores the limit, kept at or below the maximum, and returns it.
    pub fn set_min_pivot(&mut self, min_pivot: i32) -> i32 {
        self.min_pivot = min_pivot.min(self.max_pivot);
        self.min_pivot
    }

    /// Stores the limit, kept at or above the minimum, and returns it.
    pub fn set_max_pivot(&mut self, max_pivot: i32) -> i32 {
        self.max_pivot = max_pivot.max(self.min_pivot);
        self.max_pivot
    }

    /// True when a commanded servo sits outside its limits and must be moved back.
    pub fn outside_limits(&self) -> bool {
        self.commanded && !(self.min_pivot..=self.max_pivot).contains(&self.pivot())
    }
}
