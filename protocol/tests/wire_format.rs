//! Characterization tests for the Pinora wire format (audit contract C-01).
//!
//! Every command, event, registration and system message variant is serialized and
//! compared byte-for-byte with `tests/golden/wire_format.txt`, then the golden JSON is
//! deserialized and compared with the original value. The golden file records the
//! format *as it is*, including frozen misspellings (`lool_up_id`, `Ckick`, `Idol`,
//! `distant`) and the mixed tagging styles; it is not a statement of the ideal format.
//!
//! A failure here means a change altered what goes over the wire. If that is intended
//! (a versioned protocol change), regenerate the golden file with
//! `PINORA_BLESS=1 cargo test --test wire_format` and review the diff.

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::PathBuf;

use pinora_protocol::bluetooth_config::{
    DEVICE_NAME, MAX_MESSAGE_BYTES, SERVICE_NAME, SPP_WRITE_CHUNK_BYTES,
};
use pinora_protocol::*;
use serde::{Serialize, de::DeserializeOwned};

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/wire_format.txt")
}

/// Collects `name -> json` lines and round-trip checks.
#[derive(Default)]
struct Cases {
    actual: BTreeMap<String, String>,
    round_trips: Vec<Box<dyn Fn(&str) -> Result<(), String>>>,
    names: Vec<String>,
}

impl Cases {
    fn add<T>(&mut self, name: &str, value: T)
    where
        T: Serialize + DeserializeOwned + PartialEq + Debug + 'static,
    {
        let json = serde_json::to_string(&value).expect("serialize");
        assert!(
            self.actual.insert(name.to_string(), json).is_none(),
            "duplicate case name {name}"
        );
        self.names.push(name.to_string());
        self.round_trips.push(Box::new(move |golden: &str| {
            let parsed: T = serde_json::from_str(golden).map_err(|e| e.to_string())?;
            if parsed == value {
                Ok(())
            } else {
                Err(format!("round trip mismatch: {parsed:?} != {value:?}"))
            }
        }));
    }
}

fn cmd(id: &str, command: ModuleCommand) -> IncomingCommand {
    IncomingCommand {
        id: id.to_string(),
        command,
    }
}

fn event(event: ModuleEvent) -> ProtocolMessage {
    ProtocolMessage::ModuleEvent(EventPackage {
        id: "module-uuid".to_string(),
        event,
    })
}

fn all_cases() -> Cases {
    let mut c = Cases::default();

    // ---- Commands (desktop -> firmware) ----
    use LedCommandPayload as L;
    c.add(
        "cmd.led.set_state",
        cmd("u", ModuleCommand::Led(L::SetState { state: 80 })),
    );
    c.add("cmd.led.toggle", cmd("u", ModuleCommand::Led(L::Toggle {})));
    c.add(
        "cmd.reserved.ssi",
        cmd("SSI", ModuleCommand::Led(L::Toggle {})),
    );

    use ServoCommandPayload as S;
    c.add(
        "cmd.servo.set_angle",
        cmd("u", ModuleCommand::Servo(S::SetAngle { angle: -15 })),
    );
    c.add(
        "cmd.servo.set_min_pivot",
        cmd("u", ModuleCommand::Servo(S::SetMinPivot { min_pivot: -90 })),
    );
    c.add(
        "cmd.servo.set_max_pivot",
        cmd("u", ModuleCommand::Servo(S::SetMaxPivot { max_pivot: 90 })),
    );

    use LidarCommandPayload as Li;
    c.add(
        "cmd.lidar.roi",
        cmd(
            "u",
            ModuleCommand::Lidar(Li::Roi {
                min: Point { x: -45, y: -30 },
                max: Point { x: 45, y: 30 },
            }),
        ),
    );
    c.add(
        "cmd.lidar.start_scan",
        cmd("u", ModuleCommand::Lidar(Li::StartScan {})),
    );
    c.add(
        "cmd.lidar.stop_scan",
        cmd("u", ModuleCommand::Lidar(Li::StopScan {})),
    );
    c.add(
        "cmd.lidar.test",
        cmd("u", ModuleCommand::Lidar(Li::Test {})),
    );
    c.add(
        "cmd.lidar.set_step",
        cmd("u", ModuleCommand::Lidar(Li::SetStep { step: 2 })),
    );
    c.add(
        "cmd.lidar.change_motor_angle",
        cmd(
            "u",
            ModuleCommand::Lidar(Li::ChangeMotorAngle {
                id: "child".into(),
                step: 10,
            }),
        ),
    );
    c.add(
        "cmd.lidar.move_pos",
        cmd(
            "u",
            ModuleCommand::Lidar(Li::MovePos {
                p: Point { x: 1, y: 2 },
            }),
        ),
    );

    use RangefinderCommandPayload as R;
    c.add(
        "cmd.rangefinder.start_ranging",
        cmd("u", ModuleCommand::Rangefinder(R::StartRanging)),
    );
    c.add(
        "cmd.rangefinder.stop_ranging",
        cmd("u", ModuleCommand::Rangefinder(R::StopRanging)),
    );
    c.add(
        "cmd.rangefinder.set_timing_budget",
        cmd(
            "u",
            ModuleCommand::Rangefinder(R::SetTimingBudget { milliseconds: 50 }),
        ),
    );
    c.add(
        "cmd.rangefinder.set_distance_mode.short",
        cmd(
            "u",
            ModuleCommand::Rangefinder(R::SetDistanceMode {
                mode: RangefinderDistanceMode::Short,
            }),
        ),
    );
    c.add(
        "cmd.rangefinder.set_distance_mode.long",
        cmd(
            "u",
            ModuleCommand::Rangefinder(R::SetDistanceMode {
                mode: RangefinderDistanceMode::Long,
            }),
        ),
    );

    use StepperMotorCommandPayload as St;
    c.add(
        "cmd.stepper.set_pivot_min",
        cmd(
            "u",
            ModuleCommand::StepperMotor(St::SetPivotMin { pivot_min: -45.5 }),
        ),
    );
    c.add(
        "cmd.stepper.set_pivot_max",
        cmd(
            "u",
            ModuleCommand::StepperMotor(St::SetPivotMax { pivot_max: 45.5 }),
        ),
    );
    c.add(
        "cmd.stepper.move_to_origin",
        cmd("u", ModuleCommand::StepperMotor(St::MoveToOrigin {})),
    );
    c.add(
        "cmd.stepper.move_to_angle",
        cmd(
            "u",
            ModuleCommand::StepperMotor(St::MoveToAngle { angle: 90.25 }),
        ),
    );
    c.add(
        "cmd.stepper.move_to_pivot_min",
        cmd("u", ModuleCommand::StepperMotor(St::MoveToPivotMin {})),
    );
    c.add(
        "cmd.stepper.move_to_pivot_max",
        cmd("u", ModuleCommand::StepperMotor(St::MoveToPivotMax {})),
    );
    for (name, mode) in [
        ("idle", StepperStateType::Idle),
        ("moving", StepperStateType::Moving),
        ("homing", StepperStateType::Homing),
        ("pivot", StepperStateType::Pivot),
    ] {
        c.add(
            &format!("cmd.stepper.set_mode.{name}"),
            cmd("u", ModuleCommand::StepperMotor(St::SetMode { mode })),
        );
    }

    c.add(
        "cmd.rfid.write_mode",
        cmd("u", ModuleCommand::Rfid(RfidCommand::WriteMode {})),
    );
    c.add(
        "cmd.rfid.read_mode",
        cmd("u", ModuleCommand::Rfid(RfidCommand::ReadMode {})),
    );
    c.add(
        "cmd.rfid.write_payload",
        cmd(
            "u",
            ModuleCommand::Rfid(RfidCommand::WritePayload {
                data: vec![1, 2, 255],
            }),
        ),
    );

    // ---- Registration and system (firmware -> desktop) ----
    for (name, module_type) in [
        ("servo", ModuleType::Servo),
        ("led", ModuleType::Led),
        ("imu", ModuleType::Imu),
        ("led_cluster", ModuleType::LedCluster),
        ("button", ModuleType::Button),
        ("lidar", ModuleType::Lidar),
        ("rangefinder", ModuleType::Rangefinder),
        ("syslog", ModuleType::SysLog),
        ("joystick", ModuleType::JoyStick),
        ("stepper", ModuleType::StepperMotor),
        ("rfid", ModuleType::Rfid),
        ("remote_receiver", ModuleType::RemoteReceiver),
    ] {
        c.add(
            &format!("msg.registration.{name}"),
            ProtocolMessage::Registration(Registration {
                id: "module-uuid".into(),
                module_type,
                lool_up_id: "led1".into(),
                parent_id: String::new(),
            }),
        );
    }
    c.add(
        "msg.registration.with_parent",
        ProtocolMessage::Registration(Registration {
            id: "child-uuid".into(),
            module_type: ModuleType::Servo,
            lool_up_id: "servo_x".into(),
            parent_id: "lidar-uuid".into(),
        }),
    );
    c.add(
        "msg.system",
        ProtocolMessage::System(SystemInfo {
            esp_idf_version: "v5.5.3".into(),
            total_heap: "300.00 KiB (307200 bytes)".into(),
            current_free_heap: "200.00 KiB (204800 bytes)".into(),
            lowest_free_heap: "190.00 KiB (194560 bytes)".into(),
            largest_allocation: "100.00 KiB (102400 bytes)".into(),
            maximum_app_slot: "3.00 MiB (3145728 bytes)".into(),
            flash: "4.00 MiB (4194304 bytes)".into(),
        }),
    );

    // ---- Module events ----
    c.add(
        "evt.led.brightness",
        event(ModuleEvent::Led(LedEvent::Brightness { level: 80 })),
    );

    c.add(
        "evt.servo.get_angle",
        event(ModuleEvent::Servo(ServoEvent::GetAngle { angle: -15 })),
    );
    c.add(
        "evt.servo.get_min_pivot",
        event(ModuleEvent::Servo(ServoEvent::GetMinPivot {
            min_pivot: -90,
        })),
    );
    c.add(
        "evt.servo.get_max_pivot",
        event(ModuleEvent::Servo(ServoEvent::GetMaxPivot {
            max_pivot: 90,
        })),
    );
    c.add(
        "evt.servo.get_offset",
        event(ModuleEvent::Servo(ServoEvent::GetOffset { angle: 90 })),
    );

    c.add(
        "evt.lidar.roi",
        event(ModuleEvent::Lidar(LidarEvent::Roi {
            min: Point { x: -45, y: -30 },
            max: Point { x: 45, y: 30 },
        })),
    );
    c.add(
        "evt.lidar.point_map",
        event(ModuleEvent::Lidar(LidarEvent::PointMap {
            max_chunk: 3,
            curr_chunk: 1,
            map: vec![RangPoint {
                x: 1,
                y: -2,
                distant: 1234,
            }],
        })),
    );
    c.add(
        "evt.lidar.target",
        event(ModuleEvent::Lidar(LidarEvent::Target {
            point: Point { x: 3, y: 4 },
        })),
    );
    for (name, state) in [
        ("idol", ScanState::Idol),
        ("scanning", ScanState::Scanning),
        ("stop_scan", ScanState::StopScan),
    ] {
        c.add(
            &format!("evt.lidar.scan_state.{name}"),
            event(ModuleEvent::Lidar(LidarEvent::ScanState {
                state,
                scan_time: 1.5,
            })),
        );
    }

    c.add(
        "evt.button.ckick",
        event(ModuleEvent::Button(ButtonEvent::Ckick {})),
    );

    for (name, priority) in [
        ("low", LogPriority::Low),
        ("medium", LogPriority::Medium),
        ("high", LogPriority::High),
        ("critical", LogPriority::Critical),
    ] {
        c.add(
            &format!("evt.syslog.{name}"),
            event(ModuleEvent::SysLog(SysLogEvent {
                text: "message".into(),
                raw_err: None,
                priority,
            })),
        );
    }
    c.add(
        "evt.syslog.with_raw_err",
        event(ModuleEvent::SysLog(SysLogEvent {
            text: "message".into(),
            raw_err: Some("I2cError".into()),
            priority: LogPriority::Critical,
        })),
    );
    c.add(
        "evt.syslog.emitter_log_source",
        ProtocolMessage::ModuleEvent(EventPackage {
            id: "log".into(),
            event: ModuleEvent::SysLog(SysLogEvent {
                text: "t".into(),
                raw_err: None,
                priority: LogPriority::Low,
            }),
        }),
    );

    use RangefinderEvent as RE;
    c.add(
        "evt.rangefinder.range",
        event(ModuleEvent::Rangefinder(RE::Range { millimeters: 812 })),
    );
    c.add(
        "evt.rangefinder.ranging_state",
        event(ModuleEvent::Rangefinder(RE::RangingState {
            is_ranging: true,
        })),
    );
    c.add(
        "evt.rangefinder.timing_budget",
        event(ModuleEvent::Rangefinder(RE::TimingBudget {
            milliseconds: 50,
        })),
    );
    c.add(
        "evt.rangefinder.distance_mode",
        event(ModuleEvent::Rangefinder(RE::DistanceMode {
            mode: RangefinderDistanceMode::Long,
        })),
    );
    c.add(
        "evt.rangefinder.invalid_measurement",
        event(ModuleEvent::Rangefinder(RE::InvalidMeasurement {
            status: "SigmaFail".into(),
        })),
    );

    use StepperMotorEvent as SE;
    c.add(
        "evt.stepper.get_angle",
        event(ModuleEvent::StepperMotor(SE::GetAngle {
            angle: 90.0,
            step: 1024.0,
        })),
    );
    c.add(
        "evt.stepper.get_pivot_min",
        event(ModuleEvent::StepperMotor(SE::GetPivotMin {
            pivot_min: -90.0,
        })),
    );
    c.add(
        "evt.stepper.get_pivot_max",
        event(ModuleEvent::StepperMotor(SE::GetPivotMax {
            pivot_max: 90.0,
        })),
    );
    c.add(
        "evt.stepper.get_mode",
        event(ModuleEvent::StepperMotor(SE::GetMode {
            mode: StepperStateType::Homing,
        })),
    );
    c.add(
        "evt.stepper.get_origin.some",
        event(ModuleEvent::StepperMotor(SE::GetOrigin {
            origin: Some(0.5),
        })),
    );
    c.add(
        "evt.stepper.get_origin.none",
        event(ModuleEvent::StepperMotor(SE::GetOrigin { origin: None })),
    );
    c.add(
        "evt.stepper.get_pivot_point.min",
        event(ModuleEvent::StepperMotor(SE::GetPivotPoint {
            pivot_point: PivotPoint::Min,
        })),
    );
    c.add(
        "evt.stepper.get_pivot_point.max",
        event(ModuleEvent::StepperMotor(SE::GetPivotPoint {
            pivot_point: PivotPoint::Max,
        })),
    );

    let raw = RawAxes {
        x: -1,
        y: 2,
        z: 16384,
    };
    let axes = Axes {
        x: -0.5,
        y: 0.25,
        z: 1.0,
    };
    c.add(
        "evt.imu.gyro",
        event(ModuleEvent::Imu(ImuEvent::Gyro {
            raw_axes: raw,
            axes,
        })),
    );
    c.add(
        "evt.imu.accel",
        event(ModuleEvent::Imu(ImuEvent::Accel {
            raw_axes: raw,
            axes,
        })),
    );
    for (name, mode) in [
        ("collecting", MpuDeviceMode::Collecting),
        ("idle", MpuDeviceMode::Idle),
        ("off", MpuDeviceMode::Off),
    ] {
        c.add(
            &format!("evt.imu.mode.{name}"),
            event(ModuleEvent::Imu(ImuEvent::Mode { mode })),
        );
    }

    c.add(
        "evt.rfid.get_card",
        event(ModuleEvent::Rfid(RfidEvent::GetCard {
            card_uid: "[DE, AD]".into(),
            card_data: "uuid".into(),
        })),
    );
    c.add(
        "evt.rfid.get_mode.read",
        event(ModuleEvent::Rfid(RfidEvent::GetMode {
            mode: MddeRfid::Read,
        })),
    );
    c.add(
        "evt.rfid.get_mode.write",
        event(ModuleEvent::Rfid(RfidEvent::GetMode {
            mode: MddeRfid::Write,
        })),
    );
    c.add(
        "evt.rfid.get_write_state.good",
        event(ModuleEvent::Rfid(RfidEvent::GetWriteState {
            state: WriteState::Good,
            info: "All Good".into(),
        })),
    );
    c.add(
        "evt.rfid.get_write_state.bad",
        event(ModuleEvent::Rfid(RfidEvent::GetWriteState {
            state: WriteState::Bad,
            info: "err".into(),
        })),
    );

    // Every NEC command byte, so the from_command table is characterized too.
    for byte in 0..=u8::MAX {
        let key = RemoteButton::from_command(byte);
        if key != RemoteButton::None || byte == 0 {
            c.add(
                &format!("evt.remote_receiver.click.{byte:03}"),
                event(ModuleEvent::RemoteReceiver(RemoteButtonEvent::Click {
                    key,
                })),
            );
        }
    }

    c
}

fn render(actual: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (name, json) in actual {
        out.push_str(name);
        out.push('\t');
        out.push_str(json);
        out.push('\n');
    }
    out
}

#[test]
fn wire_format_matches_golden() {
    let cases = all_cases();
    let rendered = render(&cases.actual);
    let path = golden_path();

    if std::env::var_os("PINORA_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &rendered).unwrap();
        return;
    }

    let golden = std::fs::read_to_string(&path)
        .expect("golden file missing; run with PINORA_BLESS=1 once")
        .replace("\r\n", "\n");
    if golden != rendered {
        let golden_lines: BTreeMap<_, _> =
            golden.lines().filter_map(|l| l.split_once('\t')).collect();
        let mut diffs = Vec::new();
        for (name, json) in &cases.actual {
            match golden_lines.get(name.as_str()) {
                Some(g) if *g == json => {}
                Some(g) => diffs.push(format!("CHANGED {name}\n  golden: {g}\n  actual: {json}")),
                None => diffs.push(format!("NEW     {name}: {json}")),
            }
        }
        for name in golden_lines.keys() {
            if !cases.actual.contains_key(*name) {
                diffs.push(format!("MISSING {name}"));
            }
        }
        panic!("wire format changed:\n{}", diffs.join("\n"));
    }

    // Golden JSON must also deserialize back to the same value.
    let golden_lines: BTreeMap<_, _> = golden.lines().filter_map(|l| l.split_once('\t')).collect();
    for (name, check) in cases.names.iter().zip(cases.round_trips.iter()) {
        check(golden_lines[name.as_str()]).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}

#[test]
fn bluetooth_identity_and_limits_are_unchanged() {
    // Contracts C-02 and C-04: changing these breaks Windows pairings or framing limits.
    assert_eq!(DEVICE_NAME, "Pinora Dev ESP32");
    assert_eq!(SERVICE_NAME, "Pinora SPP");
    assert_eq!(MAX_MESSAGE_BYTES, 8192);
    assert_eq!(SPP_WRITE_CHUNK_BYTES, 512);
}
