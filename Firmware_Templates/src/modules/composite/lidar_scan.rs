//! Pure LiDAR scan planning (PIN-TPL-005, Phase 1D).
//!
//! A scan is a serpentine raster from the `start` corner (the ROI's `max` point) to
//! the `end` corner (the `min` point). Each row runs across x; rows advance in y.
//! Every move steps toward the current limit and is **clamped to it**, and a row (or
//! the scan) ends when the limit is reached, so every scan terminates after exactly
//! `cols × rows` positions. Before Phase 1D, termination required the position to
//! land *exactly* on the limit, so steps that did not divide the span (or `step = 0`)
//! scanned forever while the point buffer grew.
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
    position: Point,
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
            position: start.clone(),
            row_limit_x: end.x,
        })
    }

    /// The position at which the next point should be recorded.
    pub fn position(&self) -> Point {
        self.position.clone()
    }

    /// Total positions the scan visits (`cols × rows`).
    pub fn total_points(&self) -> u32 {
        positions(self.start.x, self.end.x, self.step).saturating_mul(positions(
            self.start.y,
            self.end.y,
            self.step,
        ))
    }

    /// Advances after the point at [`ScanPlan::position`] was recorded.
    pub fn advance(&mut self) -> ScanAdvance {
        let row_finished = self.position.x == self.row_limit_x;
        if row_finished && self.position.y == self.end.y {
            return ScanAdvance::Done;
        }
        if row_finished {
            // Serpentine: the next row runs back the other way.
            self.row_limit_x = if self.row_limit_x == self.end.x {
                self.start.x
            } else {
                self.end.x
            };
            self.position.y = step_toward(self.position.y, self.end.y, self.step);
            ScanAdvance::NextRow(self.position.clone())
        } else {
            self.position.x = step_toward(self.position.x, self.row_limit_x, self.step);
            ScanAdvance::NextColumn(self.position.clone())
        }
    }
}
