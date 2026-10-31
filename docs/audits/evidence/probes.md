# Probe sources (reproducibility)

These are the exact throwaway probes used for the dynamic evidence in [validation-baseline.md](validation-baseline.md). They lived **outside** the repository, in a temporary directory. To reproduce, copy each one into an empty directory outside the repo, adjust the absolute paths if the template lives elsewhere, and run the command shown. They read template sources and write only to their own directory or target dir.

## P1 — Rust (`cargo +stable run` and `cargo +stable run --release`)

`Cargo.toml`:

```toml
[package]
name = "pinora-audit-probe"
version = "0.0.0"
edition = "2021"
publish = false
[dependencies]
pinora-protocol = { path = "C:/Dev/Pinora/Pinora_Templat/protocol" }
serde_json = "1"
[workspace]
```

`src/main.rs`:

```rust
// Read-only audit probe. Includes the firmware's real math helpers and the real protocol crate.
#[path = "C:/Dev/Pinora/Pinora_Templat/Firmware_Templates/src/utilities/math.rs"]
#[allow(dead_code)]
mod math;
use pinora_protocol::*;

fn main() {
    let show = |label: &str, v: String| println!("{label}: {v}");
    // Wire-shape evidence for compatibility contracts.
    show("LED SetState", serde_json::to_string(&IncomingCommand { id: "u".into(), command: ModuleCommand::Led(LedCommandPayload::SetState { state: 80 }) }).unwrap());
    show("LED Toggle", serde_json::to_string(&IncomingCommand { id: "u".into(), command: ModuleCommand::Led(LedCommandPayload::Toggle {}) }).unwrap());
    show("Rangefinder StartRanging", serde_json::to_string(&ModuleCommand::Rangefinder(RangefinderCommandPayload::StartRanging)).unwrap());
    show("Lidar StartScan", serde_json::to_string(&ModuleCommand::Lidar(LidarCommandPayload::StartScan {})).unwrap());
    show("IMU Mode event", serde_json::to_string(&ModuleEvent::Imu(ImuEvent::Mode { mode: MpuDeviceMode::Idle })).unwrap());
    show("SysLog event", serde_json::to_string(&ModuleEvent::SysLog(SysLogEvent { text: "t".into(), raw_err: None, priority: LogPriority::Low })).unwrap());
    show("Registration", serde_json::to_string(&ProtocolMessage::Registration(Registration { id: "u".into(), module_type: ModuleType::Led, lool_up_id: "led1".into(), parent_id: String::new() })).unwrap());
    // Does the protocol accept out-of-range LED state? (No schema-level bound.)
    let big = r#"{"id":"u","command":{"Led":{"SetState":{"state":4000000000}}}}"#;
    println!("LED state=4e9 deserializes: {}", serde_json::from_str::<IncomingCommand>(big).is_ok());
    // Firmware LED mapping: range_u32(state, 0, 100, 0, 8191) (13-bit LEDC max duty).
    for state in [100u32, 101, 524_352, 524_353, 4_000_000_000] {
        let r = std::panic::catch_unwind(|| math::range_u32(state, 0, 100, 0, 8191));
        match r { Ok(v) => println!("range_u32({state}) = {v}"), Err(_) => println!("range_u32({state}) PANICS (overflow, debug build)") }
    }
}
```

## P2 — TypeScript (`bun run probe.ts`)

```ts
// Read-only probe: exercises the template's real log helpers with malformed-but-JSON input.
import { describeLog, matchesLog } from "C:/Dev/Pinora/Pinora_Templat/UI/src/lib/logs.ts";
const cases: [string, unknown][] = [
  ["valid LED event", { ModuleEvent: { id: "x", event: { Led: { Brightness: { level: 5 } } } } }],
  ["unknown top-level object", { Foo: 1 }],
  ["empty object", {}],
  ["ModuleEvent with empty event", { ModuleEvent: { id: "x", event: {} } }],
];
for (const [name, msg] of cases) {
  try {
    const d = describeLog(msg as any);
    matchesLog(msg as any, { category: "All logs", priority: "All priorities", search: "" });
    console.log(`${name}: OK -> ${d.category}`);
  } catch (e) {
    console.log(`${name}: THROWS ${(e as Error).name}: ${(e as Error).message}`);
  }
}
```
