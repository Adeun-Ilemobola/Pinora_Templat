//! Tests for `Firmware_Templates/src/modules/composite/lidar_scan.rs`
//! (Phase 1D, PIN-TPL-005): every scan terminates with bounded memory.

use pinora_firmware_host_tests::modules::composite::lidar_scan::*;
use pinora_protocol::Point;

fn p(x: i32, y: i32) -> Point {
    Point { x, y }
}

/// Runs a whole scan the way `Lidar::tick` does: record a point at the current
/// position, flush every `POINTS_PER_CHUNK`, then advance. Returns the visited
/// positions, the largest buffer size seen and the number of chunks emitted.
fn simulate(start: Point, end: Point, step: u32) -> (Vec<Point>, usize, u32, u32) {
    let mut plan = ScanPlan::new(&start, &end, step).expect("valid plan");
    let expected = plan.total_points();
    let mut visited = vec![];
    let mut buffer = 0usize;
    let mut max_buffer = 0usize;
    let mut chunks = 0u32;
    // Hard cap far above the bound, so a non-terminating plan fails instead of hanging.
    for _ in 0..(expected as usize + 10) {
        visited.push(plan.position());
        buffer += 1;
        max_buffer = max_buffer.max(buffer);
        if buffer >= POINTS_PER_CHUNK {
            buffer = 0;
            chunks += 1;
        }
        if plan.advance() == ScanAdvance::Done {
            if buffer > 0 {
                chunks += 1;
            }
            return (visited, max_buffer, chunks, expected);
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
