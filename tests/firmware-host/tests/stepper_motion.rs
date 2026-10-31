//! Tests for `Firmware_Templates/src/modules/actuators/stepper_motion.rs`
//! (Phase 1D, PIN-TPL-004): no uncommanded motion, bounded targets, coils released
//! when idle, absolute pivot targets.

use std::convert::Infallible;

use pinora_firmware_host_tests::modules::actuators::stepper_motion::*;
use pinora_protocol::module::stepper::{
    PivotPoint, StepperMotorCommandPayload as Cmd, StepperMotorEvent, StepperStateType,
};

/// Ticks with a step timer that is always due. Returns the final output of the run:
/// every event and whether the coils were released at any point.
struct Run {
    steps_taken: usize,
    events: Vec<StepperMotorEvent>,
    releases: usize,
    min_step: f32,
    max_step: f32,
}

fn run(motion: &mut StepperMotion, ticks: usize) -> Run {
    let mut result = Run {
        steps_taken: 0,
        events: Vec::new(),
        releases: 0,
        min_step: motion.step(),
        max_step: motion.step(),
    };
    for _ in 0..ticks {
        let mut stepped = false;
        let output = motion
            .tick(|_index| {
                stepped = true;
                Ok::<bool, Infallible>(true)
            })
            .unwrap();
        if stepped {
            result.steps_taken += 1;
        }
        result.releases += usize::from(output.release_coils);
        result.events.extend(output.events);
        result.min_step = result.min_step.min(motion.step());
        result.max_step = result.max_step.max(motion.step());
    }
    result
}

fn mode_event(mode: StepperStateType) -> StepperMotorEvent {
    StepperMotorEvent::GetMode { mode }
}

#[test]
fn a_new_stepper_is_idle_and_never_steps_on_its_own() {
    // PIN-TPL-004: the pre-1D motor started in Homing and swept ±90° three times.
    let mut motion = StepperMotion::new();
    assert_eq!(motion.mode(), Mode::Idle);
    let result = run(&mut motion, 50_000);
    assert_eq!(result.steps_taken, 0);
    assert!(result.events.is_empty());
    assert_eq!(motion.step(), 0.0);
}

#[test]
fn move_to_angle_reaches_the_target_then_idles_and_releases_the_coils() {
    let mut motion = StepperMotion::new();
    let output = motion.handle(&Cmd::MoveToAngle { angle: 90.0 }).unwrap();
    assert_eq!(output.events, vec![mode_event(StepperStateType::Moving)]);
    let result = run(&mut motion, 5_000);
    assert_eq!(motion.step(), 1024.0);
    assert_eq!(result.steps_taken, 1024);
    assert_eq!(motion.mode(), Mode::Idle);
    assert_eq!(result.releases, 1);
    assert_eq!(result.events, vec![mode_event(StepperStateType::Idle)]);
}

#[test]
fn holding_torque_option_keeps_the_coils_energized() {
    let mut motion = StepperMotion::new();
    motion.set_hold_when_idle(true);
    motion.handle(&Cmd::MoveToAngle { angle: -10.0 }).unwrap();
    let result = run(&mut motion, 1_000);
    assert_eq!(motion.mode(), Mode::Idle);
    assert_eq!(result.releases, 0);
}

#[test]
fn no_step_is_taken_while_the_timer_is_not_due() {
    let mut motion = StepperMotion::new();
    motion.handle(&Cmd::MoveToAngle { angle: 90.0 }).unwrap();
    for _ in 0..100 {
        motion.tick(|_| Ok::<bool, Infallible>(false)).unwrap();
    }
    assert_eq!(motion.step(), 0.0);
    assert_eq!(motion.mode(), Mode::Moving);
}

#[test]
fn targets_outside_the_travel_limit_are_rejected_without_moving() {
    for angle in [180.5, -181.0, 1e30, f32::MAX, f32::NAN, f32::INFINITY] {
        let mut motion = StepperMotion::new();
        let error = motion.handle(&Cmd::MoveToAngle { angle }).unwrap_err();
        assert!(
            matches!(error, MotionError::OutsideTravel { .. }),
            "{angle}"
        );
        assert_eq!(motion.mode(), Mode::Idle);
        assert_eq!(motion.target(), 0.0);
        assert_eq!(run(&mut motion, 100).steps_taken, 0);
    }
    // The limit itself is allowed.
    assert!(StepperMotion::new()
        .handle(&Cmd::MoveToAngle {
            angle: DEFAULT_TRAVEL_DEG
        })
        .is_ok());
}

#[test]
fn pivot_limits_are_validated_and_reported_as_applied() {
    let mut motion = StepperMotion::new();
    assert_eq!(
        motion
            .handle(&Cmd::SetPivotMax { pivot_max: 45.0 })
            .unwrap()
            .events,
        vec![StepperMotorEvent::GetPivotMax { pivot_max: 45.0 }]
    );
    assert!(matches!(
        motion
            .handle(&Cmd::SetPivotMin { pivot_min: 50.0 })
            .unwrap_err(),
        MotionError::InvertedPivotLimits { .. }
    ));
    assert!(matches!(
        motion
            .handle(&Cmd::SetPivotMax { pivot_max: -100.0 })
            .unwrap_err(),
        MotionError::InvertedPivotLimits { .. }
    ));
    assert!(matches!(
        motion
            .handle(&Cmd::SetPivotMin { pivot_min: -500.0 })
            .unwrap_err(),
        MotionError::OutsideTravel { .. }
    ));
    assert_eq!(motion.pivot_limits(), (DEFAULT_PIVOT_MIN_DEG, 45.0));
}

#[test]
fn pivot_from_a_far_position_goes_to_the_absolute_limit_without_running_away() {
    // Pre-1D, pivot targets were relative to the current position: starting at 180°,
    // the target kept moving ahead of the motor and it rotated without bound.
    let mut motion = StepperMotion::new();
    motion.handle(&Cmd::MoveToAngle { angle: 180.0 }).unwrap();
    run(&mut motion, 5_000);
    assert_eq!(motion.step(), 2048.0);

    motion.handle(&Cmd::MoveToPivotMax {}).unwrap();
    let result = run(&mut motion, 20_000);
    assert_eq!(
        motion.step(),
        1024.0,
        "settles at the absolute pivot max (90°)"
    );
    assert!(result.max_step <= 2048.0 && result.min_step >= 1024.0);
}

#[test]
fn homing_sweeps_between_the_pivot_limits_three_times_then_idles() {
    let mut motion = StepperMotion::new();
    let output = motion
        .handle(&Cmd::SetMode {
            mode: StepperStateType::Homing,
        })
        .unwrap();
    assert_eq!(output.events, vec![mode_event(StepperStateType::Homing)]);
    let result = run(&mut motion, 20_000);
    assert_eq!(motion.mode(), Mode::Idle);
    assert_eq!(result.releases, 1);
    // From 0: to Min (-1024), to Max (1024), to Min (-1024).
    assert_eq!(motion.step(), -1024.0);
    assert_eq!(result.steps_taken, 1024 + 2048 + 2048);
    assert!(result.min_step >= -1024.0 && result.max_step <= 1024.0);
    let points: Vec<_> = result
        .events
        .iter()
        .filter_map(|e| match e {
            StepperMotorEvent::GetPivotPoint { pivot_point } => Some(*pivot_point),
            _ => None,
        })
        .collect();
    assert_eq!(
        points,
        vec![PivotPoint::Min, PivotPoint::Max, PivotPoint::Min]
    );
}

#[test]
fn move_to_origin_completes_the_whole_move() {
    // Pre-1D, MoveToOrigin took at most one step unless the motor was already Moving.
    let mut motion = StepperMotion::new();
    motion.handle(&Cmd::MoveToAngle { angle: -45.0 }).unwrap();
    run(&mut motion, 2_000);
    motion.handle(&Cmd::MoveToOrigin {}).unwrap();
    run(&mut motion, 2_000);
    assert_eq!(motion.step(), 0.0);
    assert_eq!(motion.mode(), Mode::Idle);
}

#[test]
fn set_mode_idle_stops_in_place_and_releases() {
    let mut motion = StepperMotion::new();
    motion.handle(&Cmd::MoveToAngle { angle: 90.0 }).unwrap();
    run(&mut motion, 100);
    let output = motion
        .handle(&Cmd::SetMode {
            mode: StepperStateType::Idle,
        })
        .unwrap();
    assert!(output.release_coils);
    assert_eq!(motion.target(), motion.step());
    assert_eq!(run(&mut motion, 1_000).steps_taken, 0);
    // Re-entering Moving does not resume the abandoned target.
    motion
        .handle(&Cmd::SetMode {
            mode: StepperStateType::Moving,
        })
        .unwrap();
    assert_eq!(run(&mut motion, 1_000).steps_taken, 0);
}

#[test]
fn coil_errors_propagate_and_the_position_is_unchanged() {
    let mut motion = StepperMotion::new();
    motion.handle(&Cmd::MoveToAngle { angle: 1.0 }).unwrap();
    let result = motion.tick(|_| Err::<bool, &str>("GPIO write failed"));
    assert_eq!(result.unwrap_err(), "GPIO write failed");
    assert_eq!(motion.step(), 0.0);
}

#[test]
fn half_step_sequence_wraps_in_both_directions() {
    assert_eq!(sequence_index(0.0), 0);
    assert_eq!(sequence_index(1.0), 1);
    assert_eq!(sequence_index(8.0), 0);
    assert_eq!(sequence_index(-1.0), 7);
    assert_eq!(angle_to_step(90.0), 1024.0);
    assert_eq!(step_to_angle(2048.0), 180.0);
}
