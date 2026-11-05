//! Tests for `Firmware_Templates/src/modules/composite/lidar_scan.rs`
//! (Phase 1D, PIN-TPL-005): every scan terminates with bounded memory, and the next
//! position always comes from the position read back from the servos.

use pinora_firmware_host_tests::modules::composite::lidar_scan::*;
use pinora_protocol::Point;

fn p(x: i32, y: i32) -> Point {
    Point { x, y }
}

/// Stands in for the two servos: the read-back after a move is the commanded angle
/// after each axis's limits, as `ServoModule::pivot_angle` gives it.
#[derive(Clone, Copy)]
struct FakeServos {
    x: (i32, i32),
    y: (i32, i32),
}

const FULL_RANGE: FakeServos = FakeServos {
    x: (-90, 90),
    y: (-90, 90),
};

impl FakeServos {
    fn move_to(&self, target: &Point) -> Point {
        p(
            target.x.clamp(self.x.0, self.x.1),
            target.y.clamp(self.y.0, self.y.1),
        )
    }
}

/// Runs a scan on `servos` and returns every position recorded.
fn scan_on(servos: FakeServos, start: Point, end: Point, step: u32) -> Vec<Point> {
    let mut plan = ScanPlan::new(&start, &end, step).expect("valid plan");
    let mut current = servos.move_to(&plan.start());
    let mut visited = vec![];
    for _ in 0..200_000 {
        visited.push(current.clone());
        match plan.advance(&current) {
            ScanAdvance::Done => return visited,
            ScanAdvance::NextColumn(next) | ScanAdvance::NextRow(next) => {
                current = servos.move_to(&next)
            }
        }
    }
    panic!("scan {start:?} -> {end:?} step {step} did not terminate");
}

/// Runs a whole scan the way `Lidar::tick` does: record a point at the current
/// position, flush every `POINTS_PER_CHUNK`, then advance. Returns the visited
/// positions, the largest buffer size seen and the number of chunks emitted.
fn simulate(start: Point, end: Point, step: u32) -> (Vec<Point>, usize, u32, u32) {
    let mut plan = ScanPlan::new(&start, &end, step).expect("valid plan");
    let expected = plan.total_points();
    let mut current = FULL_RANGE.move_to(&plan.start());
    let mut visited = vec![];
    let mut buffer = 0usize;
    let mut max_buffer = 0usize;
    let mut chunks = 0u32;
    // Hard cap far above the bound, so a non-terminating plan fails instead of hanging.
    for _ in 0..(expected as usize + 10) {
        visited.push(current.clone());
        buffer += 1;
        max_buffer = max_buffer.max(buffer);
        if buffer >= POINTS_PER_CHUNK {
            buffer = 0;
            chunks += 1;
        }
        match plan.advance(&current) {
            ScanAdvance::Done => {
                if buffer > 0 {
                    chunks += 1;
                }
                return (visited, max_buffer, chunks, expected);
            }
            ScanAdvance::NextColumn(next) | ScanAdvance::NextRow(next) => {
                current = FULL_RANGE.move_to(&next)
            }
        }
    }
    panic!("scan {start:?} -> {end:?} step {step} did not terminate within {expected} points");
}

#[test]
fn every_roi_and_step_terminates_with_exact_counts_and_bounded_memory() {
    let coords = [-90, -77, -50, -1, 0, 3, 45, 88, 90];
    let steps = [1, 2, 3, 7, 13, 45, 90, 180];
    let mut scans = 0;
    for &sx in &coords {
        for &sy in &coords {
            for &(ex, ey) in &[(90, -90), (-90, 90), (sx, sy), (-50, 7), (0, 0)] {
                for &step in &steps {
                    let (start, end) = (p(sx, sy), p(ex, ey));
                    let (visited, max_buffer, chunks, expected) =
                        simulate(start.clone(), end.clone(), step);
                    assert_eq!(visited.len() as u32, expected);
                    assert!(max_buffer <= POINTS_PER_CHUNK);
                    assert_eq!(chunks, chunk_count(expected));
                    assert_eq!(visited.first(), Some(&start));
                    // Every position stays inside the ROI bounding box.
                    let (x0, x1) = (sx.min(ex), sx.max(ex));
                    let (y0, y1) = (sy.min(ey), sy.max(ey));
                    assert!(visited
                        .iter()
                        .all(|v| (x0..=x1).contains(&v.x) && (y0..=y1).contains(&v.y)));
                    // The scan ends on the end row, at one of its x limits.
                    let last = visited.last().unwrap();
                    assert_eq!(last.y, ey);
                    assert!(last.x == ex || last.x == sx);
                    scans += 1;
                }
            }
        }
    }
    assert!(scans > 3_000, "{scans}");
}

#[test]
fn the_case_from_the_finding_now_terminates() {
    // PIN-TPL-005: x from 90 to -50 with step 3 never landed on -50, so the old scan
    // ran on, clamped at -90, forever.
    let (visited, _, _, _) = simulate(p(90, 10), p(-50, 0), 3);
    assert!(
        visited.iter().any(|v| v.x == -50),
        "the limit itself is visited"
    );
}

#[test]
fn the_default_full_scan_has_the_expected_size() {
    // Lidar defaults: start (max) = (-90, 90), end (min) = (90, -90), step 1.
    let plan = ScanPlan::new(&p(-90, 90), &p(90, -90), 1).unwrap();
    assert_eq!(plan.total_points(), 181 * 181);
    assert_eq!(chunk_count(plan.total_points()), 328);
}

#[test]
fn rows_alternate_direction() {
    let (visited, ..) = simulate(p(0, 2), p(2, 0), 1);
    let xs: Vec<_> = visited.iter().map(|v| (v.x, v.y)).collect();
    assert_eq!(
        xs,
        vec![
            (0, 2),
            (1, 2),
            (2, 2),
            (2, 1),
            (1, 1),
            (0, 1),
            (0, 0),
            (1, 0),
            (2, 0)
        ]
    );
}

#[test]
fn the_next_position_comes_from_the_position_it_is_given() {
    let mut plan = ScanPlan::new(&p(0, 5), &p(10, 0), 2).unwrap();
    assert_eq!(plan.advance(&p(0, 5)), ScanAdvance::NextColumn(p(2, 5)));
    // Not where the plan sent it: a command moved the servo to (7, 3) during the scan.
    assert_eq!(plan.advance(&p(7, 3)), ScanAdvance::NextColumn(p(9, 3)));
    assert_eq!(plan.advance(&p(9, 3)), ScanAdvance::NextColumn(p(10, 3)));
    assert_eq!(plan.advance(&p(10, 3)), ScanAdvance::NextRow(p(10, 1)));
}

#[test]
fn a_servo_moved_during_a_scan_is_followed_to_the_end() {
    // Moved past the end row and outside the ROI: the scan still comes back and ends.
    let mut plan = ScanPlan::new(&p(0, 2), &p(2, 0), 1).unwrap();
    let _ = plan.advance(&p(0, 2));
    let mut current = p(-5, -3);
    for _ in 0..50 {
        match plan.advance(&current) {
            ScanAdvance::Done => {
                assert_eq!(current.y, 0);
                return;
            }
            ScanAdvance::NextColumn(next) | ScanAdvance::NextRow(next) => current = next,
        }
    }
    panic!("did not terminate");
}

#[test]
fn a_clamped_x_servo_ends_each_row_at_its_limit() {
    let servos = FakeServos {
        x: (-3, 90),
        ..FULL_RANGE
    };
    let visited = scan_on(servos, p(0, 1), p(-5, 0), 2);
    // -2, then -4 is clamped to -3, then a move to -5 leaves it at -3: end of the row.
    // That last attempt records (-3, 1) a second time.
    assert_eq!(
        visited,
        vec![
            p(0, 1),
            p(-2, 1),
            p(-3, 1),
            p(-3, 1),
            p(-3, 0),
            p(-1, 0),
            p(0, 0)
        ]
    );
}

#[test]
fn a_clamped_y_servo_ends_the_scan() {
    let servos = FakeServos {
        y: (1, 90),
        ..FULL_RANGE
    };
    let visited = scan_on(servos, p(0, 2), p(1, 0), 1);
    // The move from row 1 to row 0 leaves y at 1: the scan ends instead of repeating.
    assert_eq!(visited.last().map(|v| v.y), Some(1));
    assert!(visited.len() <= 6, "{visited:?}");
}

#[test]
fn a_servo_that_never_moves_ends_the_scan_quickly() {
    let stuck = FakeServos {
        x: (5, 5),
        y: (5, 5),
    };
    let visited = scan_on(stuck, p(-90, 90), p(90, -90), 1);
    assert!(visited.iter().all(|v| *v == p(5, 5)));
    assert!(visited.len() <= 3, "{visited:?}");
}

#[test]
fn every_clamped_scan_terminates_and_records_only_reachable_positions() {
    let corners = [(-90, 90), (90, -90), (-50, 7), (3, 3), (0, -77)];
    let limits = [(-90, 90), (-30, 30), (0, 90), (-90, 0), (5, 5), (-1, 1)];
    let steps = [1, 3, 7, 45, 180];
    let mut scans = 0;
    for &(sx, sy) in &corners {
        for &(ex, ey) in &corners {
            for &x in &limits {
                for &y in &limits {
                    for &step in &steps {
                        let visited = scan_on(FakeServos { x, y }, p(sx, sy), p(ex, ey), step);
                        assert!(visited
                            .iter()
                            .all(|v| { (x.0..=x.1).contains(&v.x) && (y.0..=y.1).contains(&v.y) }));
                        scans += 1;
                    }
                }
            }
        }
    }
    assert_eq!(scans, 5 * 5 * 6 * 6 * 5);
}

#[test]
fn invalid_steps_are_rejected() {
    assert_eq!(validate_step(0), Err(ScanError::InvalidStep { step: 0 }));
    assert_eq!(
        validate_step(181),
        Err(ScanError::InvalidStep { step: 181 })
    );
    assert!(validate_step(1).is_ok() && validate_step(MAX_STEP).is_ok());
    assert!(ScanPlan::new(&p(0, 0), &p(1, 1), 0).is_err());
}

#[test]
fn roi_corners_outside_the_servo_range_are_rejected() {
    for (min, max) in [
        (p(-91, 0), p(0, 0)),
        (p(0, 0), p(0, 91)),
        (p(i32::MIN, 0), p(0, 0)),
        (p(0, i32::MAX), p(0, 0)),
    ] {
        assert!(matches!(
            validate_roi(&min, &max),
            Err(ScanError::RoiOutOfRange { .. })
        ));
    }
    assert!(validate_roi(&p(-90, -90), &p(90, 90)).is_ok());
}

#[test]
fn error_messages_are_readable() {
    assert_eq!(
        ScanError::InvalidStep { step: 0 }.to_string(),
        "LiDAR scan step 0 is outside 1-180"
    );
    assert_eq!(
        ScanError::RoiOutOfRange { point: (95, 0) }.to_string(),
        "LiDAR ROI corner (95, 0) is outside the servo range ±90°"
    );
}

#[test]
fn a_measurement_counts_only_if_it_began_after_the_servos_arrived() {
    // A 1 degree step with a 50 ms budget: 4 ms to arrive, then a whole measurement.
    assert_eq!(reading_usable_from_us(1_000, 1, 50), 1_000 + 54_000);
    // A long move waits for the travel as well.
    assert_eq!(reading_usable_from_us(0, 180, 50), 770_000);
    // No move still needs a whole measurement.
    assert_eq!(reading_usable_from_us(5, 0, 100), 100_005);
}

#[test]
fn travel_is_the_larger_of_the_two_axes() {
    assert_eq!(travel_degrees(&p(-90, 0), &p(90, 5)), 180);
    assert_eq!(travel_degrees(&p(3, -90), &p(4, 90)), 180);
    assert_eq!(travel_degrees(&p(7, 7), &p(7, 7)), 0);
}

#[test]
fn the_reading_timeout_outlasts_the_longest_measurement() {
    const LONGEST_TIMING_BUDGET_US: u64 = 500_000;
    const { assert!(READING_TIMEOUT_US > 2 * LONGEST_TIMING_BUDGET_US) };
}
