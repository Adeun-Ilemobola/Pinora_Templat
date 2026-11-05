//! Tests for `Firmware_Templates/src/modules/actuators/servo_position.rs`: the reported
//! position must be where the servo was last really sent.

use pinora_firmware_host_tests::modules::actuators::servo_position::ServoPosition;
use pinora_protocol::module::servomodule::ServoCapability;

fn lidar_config() -> ServoCapability {
    ServoCapability {
        max_angle: 180,
        min_angle: 0,
        offset: 90,
        max_pivot: 90,
        min_pivot: -90,
        pulse_max: 2500,
        pulse_min: 500,
    }
}

fn ok(_pulse: i32) -> Result<(), ()> {
    Ok(())
}
fn fail(_pulse: i32) -> Result<(), ()> {
    Err(())
}

#[test]
fn a_new_servo_is_not_claimed_to_be_anywhere_yet() {
    assert!(!ServoPosition::new(&lidar_config()).commanded());
}

#[test]
fn a_failed_write_leaves_the_position_where_the_servo_really_is() {
    let mut servo = ServoPosition::new(&lidar_config());
    servo.move_to(30, ok).unwrap();
    assert!(servo.move_to(60, fail).is_err());
    assert_eq!(servo.pivot(), 30);
    assert_eq!(servo.angle(), 120);
}

#[test]
fn a_first_write_that_fails_does_not_count_as_commanded() {
    let mut servo = ServoPosition::new(&lidar_config());
    assert!(servo.move_to(60, fail).is_err());
    assert!(!servo.commanded());
    assert_eq!(servo.pivot(), 0);
}

#[test]
fn the_pulse_written_matches_the_position_saved() {
    let mut servo = ServoPosition::new(&lidar_config());
    let mut written = 0;
    servo
        .move_to(-90, |pulse| {
            written = pulse;
            ok(pulse)
        })
        .unwrap();
    assert_eq!(written, 500);
    assert_eq!(servo.pivot(), -90);
    servo
        .move_to(500, |pulse| {
            written = pulse;
            ok(pulse)
        })
        .unwrap();
    assert_eq!(written, 2500);
    assert_eq!(servo.pivot(), 90, "reports the limit, not the request");
}

#[test]
fn an_off_centre_offset_reports_the_angle_after_the_hardware_limit() {
    // Offset 150: pivot 90 would be 240, but the servo stops at 180.
    let mut servo = ServoPosition::new(&ServoCapability {
        offset: 150,
        ..lidar_config()
    });
    servo.move_to(90, ok).unwrap();
    assert_eq!(servo.angle(), 180);
    assert_eq!(servo.pivot(), 30, "where it is, not the 90 that was asked for");
}

#[test]
fn changing_the_offset_keeps_position_and_pivot_true() {
    let mut servo = ServoPosition::new(&lidar_config());
    servo.move_to(30, ok).unwrap();
    servo.move_to_new_offset(100, ok).unwrap();
    assert_eq!(servo.angle(), 100, "the servo was driven to the new offset");
    assert_eq!(servo.pivot(), 0);
    assert!(servo.move_to_new_offset(50, fail).is_err());
    assert_eq!(servo.offset(), 100);
    assert_eq!(servo.angle(), 100);
}

#[test]
fn narrowing_the_limits_flags_a_servo_left_outside_them() {
    let mut servo = ServoPosition::new(&lidar_config());
    assert_eq!(servo.set_max_pivot(20), 20);
    assert_eq!(servo.set_min_pivot(5), 5);
    assert!(
        !servo.outside_limits(),
        "not commanded yet: pivot 0 is outside, but nothing is known to move back"
    );
    assert_eq!(servo.set_min_pivot(-90), -90);
    servo.move_to(10, ok).unwrap();
    assert!(!servo.outside_limits());
    assert_eq!(servo.set_max_pivot(5), 5);
    assert!(servo.outside_limits());
    servo.move_to(servo.pivot(), ok).unwrap();
    assert_eq!(servo.pivot(), 5);
    assert!(!servo.outside_limits());
}

#[test]
fn limits_cannot_cross() {
    let mut servo = ServoPosition::new(&lidar_config());
    assert_eq!(servo.set_max_pivot(10), 10);
    assert_eq!(servo.set_min_pivot(50), 10);
    assert_eq!(servo.set_max_pivot(-5), 10);
    assert_eq!(servo.pivot_limits(), (10, 10));
}
