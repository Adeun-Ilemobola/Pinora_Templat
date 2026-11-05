//! Pure LiDAR scan planning (PIN-TPL-005, Phase 1D).
//!
//! A scan is a serpentine raster from the `start` corner (the ROI's `max` point) to
//! the `end` corner (the `min` point). Each row runs across x; rows advance in y.
//!
//! The plan holds **no position of its own**: [`ScanPlan::advance`] is given the
//! position read back from the servos and works out the next point from it. So a
//! servo that is clamped, or moved by a command during a scan, is followed rather
//! than ignored. (The servos have no feedback: the read-back is the last angle they
//! were commanded to, after their limits.)
//!
//! Every scan terminates, for two reasons. A move steps toward the current limit and
//! is **clamped to it**, so steps that do not divide the span still land on it (before
//! Phase 1D such a scan ran forever while the point buffer grew). And a servo that
//! stayed put although a move was requested cannot get any further, so that ends the
//! row (x) or the scan (y). With servos that reach every target, a scan visits exactly
//! `cols × rows` positions.
//!
//! No ESP-IDF types, so this is tested on the host (`tests/firmware-host`).

use std::fmt;

use pinora_protocol::Point;

/// Points per `PointMap` chunk; also the maximum size of the point buffer.
pub const POINTS_PER_CHUNK: usize = 100;
/// The LiDAR servos' pivot range (degrees either side of their offset).
pub const PIVOT_LIMIT: i32 = 90;
/// Largest useful step: the full pivot span.
pub const MAX_STEP: u32 = 2 * PIVOT_LIMIT as u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    /// A ROI corner outside `-PIVOT_LIMIT..=PIVOT_LIMIT` on either axis.
    RoiOutOfRange { point: (i32, i32) },
    /// A step outside `1..=MAX_STEP`.
    InvalidStep { step: u32 },
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::RoiOutOfRange { point: (x, y) } => write!(
                f,
                "LiDAR ROI corner ({x}, {y}) is outside the servo range ±{PIVOT_LIMIT}°"
            ),
            ScanError::InvalidStep { step } => {
                write!(f, "LiDAR scan step {step} is outside 1-{MAX_STEP}")
            }
        }
    }
}

impl std::error::Error for ScanError {}

pub fn validate_roi(min: &Point, max: &Point) -> Result<(), ScanError> {
    for point in [min, max] {
        let in_range = |v: i32| (-PIVOT_LIMIT..=PIVOT_LIMIT).contains(&v);
        if !in_range(point.x) || !in_range(point.y) {
            return Err(ScanError::RoiOutOfRange {
                point: (point.x, point.y),
            });
        }
    }
    Ok(())
}

pub fn validate_step(step: u32) -> Result<(), ScanError> {
    if (1..=MAX_STEP).contains(&step) {
        Ok(())
    } else {
        Err(ScanError::InvalidStep { step })
    }
}

/// Positions along one axis from `a` to `b` inclusive with the given step (the last
/// move is clamped to `b`).
fn positions(a: i32, b: i32, step: u32) -> u32 {
    a.abs_diff(b).div_ceil(step) + 1
}

/// Number of chunks for `points` points (at least 1 for a non-empty scan).
pub fn chunk_count(points: u32) -> u32 {
    points.div_ceil(POINTS_PER_CHUNK as u32)
}

/// Moves `from` toward `to` by `step`, without passing `to`.
fn step_toward(from: i32, to: i32, step: u32) -> i32 {
    let step = step as i64;
    let next = if from < to {
        (from as i64 + step).min(to as i64)
    } else {
        (from as i64 - step).max(to as i64)
    };
    next as i32
}

/// What the scan does after the point at the current position was recorded.
#[derive(Debug, Clone, PartialEq)]
pub enum ScanAdvance {
    /// Move along the current row to this position.
    NextColumn(Point),
    /// The row is complete; move to this position, the first of the next row.
    NextRow(Point),
    /// The last position was recorded; the scan is complete.
    Done,
}

#[derive(Debug, Clone)]
pub struct ScanPlan {
    start: Point,
    end: Point,
    step: u32,
    /// Where the servos were when the last move was requested.
    from: Point,
    /// Where the last move was going.
    target: Point,
    row_limit_x: i32,
}

impl ScanPlan {
    /// Plans a scan from `start` to `end`. Both are validated, as is `step`.
    pub fn new(start: &Point, end: &Point, step: u32) -> Result<Self, ScanError> {
        validate_roi(end, start)?;
        validate_step(step)?;
        Ok(Self {
            start: start.clone(),
            end: end.clone(),
            step,
            from: start.clone(),
            target: start.clone(),
            row_limit_x: end.x,
        })
    }

    /// Where the scan begins.
    pub fn start(&self) -> Point {
        self.start.clone()
    }

    /// Total positions the scan visits (`cols × rows`) when the servos reach every
    /// target. Clamped servos change this: it can then be off in either direction.
    pub fn total_points(&self) -> u32 {
        positions(self.start.x, self.end.x, self.step).saturating_mul(positions(
            self.start.y,
            self.end.y,
            self.step,
        ))
    }

    /// Decides the next move, given `current`: the position read back from the servos,
    /// where the last point was recorded.
    pub fn advance(&mut self, current: &Point) -> ScanAdvance {
        // It stayed where it was although a move was requested: it cannot go further.
        let stalled = |now: i32, from: i32, target: i32| now == from && now != target;
        if stalled(current.y, self.from.y, self.target.y) {
            return ScanAdvance::Done;
        }
        let row_finished =
            current.x == self.row_limit_x || stalled(current.x, self.from.x, self.target.x);
        if row_finished && current.y == self.end.y {
            return ScanAdvance::Done;
        }

        self.from = current.clone();
        self.target = current.clone();
        if row_finished {
            // Serpentine: the next row runs back the other way.
            self.row_limit_x = if self.row_limit_x == self.end.x {
                self.start.x
            } else {
                self.end.x
            };
            self.target.y = step_toward(current.y, self.end.y, self.step);
            ScanAdvance::NextRow(self.target.clone())
        } else {
            self.target.x = step_toward(current.x, self.row_limit_x, self.step);
            ScanAdvance::NextColumn(self.target.clone())
        }
    }
}
