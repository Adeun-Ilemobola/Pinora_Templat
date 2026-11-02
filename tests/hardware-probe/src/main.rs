//! On-device checks for the Pinora firmware (audit Phase 2B, B2-V7). Talks to the
//! board over its USB console and Bluetooth SPP with the real protocol crate,
//! including the HMAC handshake. Never prints a device key; leaves the device
//! unprovisioned. Drives only the three LEDs of the default image.
//!
//! Usage:
//!   pinora-hardware-probe boot <usb> [secs]     reset the board and print its console
//!   pinora-hardware-probe scenario <usb> <bt>   protocol v2 scenario, PASS/FAIL lines
//!   pinora-hardware-probe persist <usb>         the key survives a reset (NVS)

use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use pinora_protocol::auth::{AuthStatus, ClientSession, Key, NONCE_BYTES};
use pinora_protocol::framing::{Frame, LineFramer, MAX_LINE_BYTES};
use pinora_protocol::*;

struct Port {
    name: String,
    writer: Box<dyn serialport::SerialPort>,
    lines: Receiver<String>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    reader: Option<thread::JoinHandle<()>>,
}

impl Drop for Port {
    fn drop(&mut self) {
        // The reader owns a clone of the handle; the port closes only when it ends.
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

impl Port {
    fn open(name: &str, reset: bool) -> Result<Port, String> {
        let mut port = serialport::new(name, 115_200)
            .timeout(Duration::from_millis(20))
            .open()
            .map_err(|e| format!("open {name}: {e}"))?;
        if reset {
            // Same sequence as the desktop's USB connect: may reset the board.
            let _ = port.write_data_terminal_ready(false);
            let _ = port.write_request_to_send(true);
            thread::sleep(Duration::from_millis(100));
            let _ = port.write_request_to_send(false);
        }
        let mut reader = port.try_clone().map_err(|e| e.to_string())?;
        let (tx, rx) = channel();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let reader_stop = stop.clone();
        let handle = thread::spawn(move || {
            let mut framer = LineFramer::new(MAX_LINE_BYTES);
            let mut buf = [0u8; 1024];
            while !reader_stop.load(std::sync::atomic::Ordering::Relaxed) {
                match reader.read(&mut buf) {
                    Ok(n) => {
                        for f in framer.push(&buf[..n]) {
                            if let Frame::Line(l) = f {
                                if tx.send(String::from_utf8_lossy(&l).into_owned()).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(_) => return,
                }
            }
        });
        Ok(Port {
            name: name.into(),
            writer: port,
            lines: rx,
            stop,
            reader: Some(handle),
        })
    }

    fn send_line(&mut self, line: &str) {
        self.writer.write_all(line.as_bytes()).unwrap();
        self.writer.write_all(b"\n").unwrap();
        self.writer.flush().unwrap();
    }

    fn send<T: serde::Serialize>(&mut self, value: &T) -> String {
        let line = serde_json::to_string(value).unwrap();
        self.send_line(&line);
        line
    }

    /// Waits for a protocol message matching `pred`; collects everything seen.
    fn expect<F: Fn(&ProtocolMessage) -> bool>(
        &self,
        secs: f32,
        pred: F,
    ) -> (Option<ProtocolMessage>, Vec<String>) {
        let deadline = Instant::now() + Duration::from_secs_f32(secs);
        let mut seen = Vec::new();
        while let Some(left) = deadline.checked_duration_since(Instant::now()) {
            match self.lines.recv_timeout(left) {
                Ok(line) => {
                    if let Ok(msg) = serde_json::from_str::<ProtocolMessage>(line.trim()) {
                        if pred(&msg) {
                            return (Some(msg), seen);
                        }
                    }
                    seen.push(line);
                }
                Err(_) => break,
            }
        }
        (None, seen)
    }

    fn drain(&self, secs: f32) -> Vec<String> {
        self.expect(secs, |_| false).1
    }
}

fn cmd(id: &str, command: ModuleCommand) -> IncomingCommand {
    IncomingCommand {
        id: id.into(),
        command,
    }
}
fn sys(command: SystemCommand) -> IncomingCommand {
    cmd(SYSTEM_COMMAND_ID, ModuleCommand::System(command))
}
fn set(id: &str, state: u32) -> IncomingCommand {
    cmd(
        id,
        ModuleCommand::Led(LedCommandPayload::SetState { state }),
    )
}
fn nonce() -> [u8; NONCE_BYTES] {
    let mut n = [0u8; NONCE_BYTES];
    getrandom::getrandom(&mut n).unwrap();
    n
}

fn is_brightness(msg: &ProtocolMessage, id: &str, level: u32) -> bool {
    matches!(msg, ProtocolMessage::ModuleEvent(EventPackage { id: i, event: ModuleEvent::Led(LedEvent::Brightness { level: l }) }) if i == id && *l == level)
}
fn syslog_text(msg: &ProtocolMessage) -> Option<(String, String, LogPriority)> {
    match msg {
        ProtocolMessage::ModuleEvent(EventPackage {
            id,
            event: ModuleEvent::SysLog(s),
        }) => Some((id.clone(), s.text.clone(), s.priority.clone())),
        _ => None,
    }
}

struct Report {
    pass: u32,
    fail: u32,
}
impl Report {
    fn check(&mut self, name: &str, ok: bool, detail: &str) {
        if ok {
            self.pass += 1;
            println!("PASS  {name}");
        } else {
            self.fail += 1;
            println!("FAIL  {name}  -- {detail}");
        }
    }
}

fn boot(usb: &str, secs: f32) {
    let port = Port::open(usb, true).unwrap();
    for line in port.drain(secs) {
        println!("[{}] {line}", port.name);
    }
}

/// Registers: lookup name -> module id, from a Rediscover burst.
fn discover(bt: &mut Port) -> (Option<SystemInfo>, Vec<(String, String)>, Vec<String>) {
    bt.send(&sys(SystemCommand::Rediscover {}));
    let mut info = None;
    let mut leds = Vec::new();
    let mut raw = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(4);
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        let Ok(line) = bt.lines.recv_timeout(left) else {
            break;
        };
        match serde_json::from_str::<ProtocolMessage>(line.trim()) {
            Ok(ProtocolMessage::System(s)) => info = Some(s),
            Ok(ProtocolMessage::Registration(r))
                if r.module_type == ModuleType::Led
                    && !leds
                        .iter()
                        .any(|(n, _): &(String, String)| *n == r.lool_up_id) =>
            {
                leds.push((r.lool_up_id.clone(), r.id.clone()));
            }
            _ => {}
        }
        raw.push(line);
        if leds.len() == 3 && info.is_some() && raw.len() > 8 {
            // Let the state replay finish.
            raw.extend(bt.drain(0.5));
            break;
        }
    }
    leds.sort();
    (info, leds, raw)
}

fn scenario(usb: &str, bt_name: &str) {
    let mut r = Report { pass: 0, fail: 0 };
    let mut usb = Port::open(usb, false).unwrap();

    // Start from an unprovisioned device.
    usb.send(&sys(SystemCommand::ClearKey {}));
    let (cleared, _) = usb.expect(3.0, |m| {
        matches!(m, ProtocolMessage::Auth(AuthMessage::KeyCleared { .. }))
    });
    let device_id = match &cleared {
        Some(ProtocolMessage::Auth(AuthMessage::KeyCleared { device_id })) => device_id.clone(),
        _ => String::new(),
    };
    r.check(
        "USB console answers ClearKey (console runs in the Bluetooth image)",
        cleared.is_some(),
        "no KeyCleared on USB",
    );
    println!("      device id {device_id}");

    println!("      opening {bt_name} (Bluetooth SPP)...");
    let mut bt = match Port::open(bt_name, false) {
        Ok(p) => p,
        Err(e) => {
            r.check("Bluetooth SPP opens", false, &e);
            println!("\nRESULT {} passed, {} failed", r.pass, r.fail);
            return;
        }
    };
    thread::sleep(Duration::from_millis(500));

    // Unprovisioned handshake.
    let mut client = ClientSession::new();
    bt.send(&client.begin(nonce()));
    let (m, seen) = bt.expect(3.0, |m| matches!(m, ProtocolMessage::Auth(_)));
    if let Some(ProtocolMessage::Auth(a)) = &m {
        client.on_message(a, |_| None);
    }
    r.check(
        "AuthBegin on an unprovisioned device -> Unprovisioned",
        client.status()
            == AuthStatus::Unprovisioned {
                device_id: device_id.clone(),
            },
        &format!("{m:?} {seen:?}"),
    );

    // Rediscovery and replay.
    let (info, leds, raw) = discover(&mut bt);
    r.check(
        "Rediscover returns a v2 system snapshot",
        info.as_ref().map(|i| i.protocol_version) == Some(2),
        &format!("{info:?}"),
    );
    r.check(
        "Rediscover returns 3 LED registrations",
        leds.len() == 3,
        &format!("{leds:?} {raw:?}"),
    );
    let replayed = raw.iter().filter(|l| l.contains("Brightness")).count();
    r.check(
        "Rediscover replays LED brightness (first_emit)",
        replayed >= 3,
        &format!("{replayed} Brightness lines"),
    );
    if leds.len() != 3 {
        println!("\nRESULT {} passed, {} failed", r.pass, r.fail);
        return;
    }
    let ids: Vec<String> = leds.iter().map(|(_, id)| id.clone()).collect();

    // Unprovisioned commands work, with one warning.
    bt.send(&set(&ids[0], 30));
    let (m, seen) = bt.expect(2.0, |m| is_brightness(m, &ids[0], 30));
    let warned = seen.iter().any(|l| l.contains("no key"));
    r.check(
        "Unprovisioned: SetState 30 is applied",
        m.is_some(),
        &format!("{seen:?}"),
    );
    r.check(
        "Unprovisioned: one Medium warning that commands are not authenticated",
        warned,
        &format!("{seen:?}"),
    );

    // 1B clamp.
    bt.send(&set(&ids[0], 250));
    let (m, seen) = bt.expect(2.0, |m| is_brightness(m, &ids[0], 100));
    r.check(
        "SetState 250 is clamped to 100 (1B)",
        m.is_some(),
        &format!("{seen:?}"),
    );

    // Pair over USB.
    usb.send(&sys(SystemCommand::ProvisionKey {}));
    let (m, seen) = usb.expect(3.0, |m| {
        matches!(m, ProtocolMessage::Auth(AuthMessage::Key { .. }))
    });
    let key = match &m {
        Some(ProtocolMessage::Auth(AuthMessage::Key { key, .. })) => Key::from_hex(key),
        _ => None,
    };
    r.check(
        "USB ProvisionKey returns a 256-bit key",
        key.is_some(),
        &format!("{} other lines", seen.len()),
    );
    let Some(key) = key else {
        println!("\nRESULT {} passed, {} failed", r.pass, r.fail);
        return;
    };
    usb.send(&sys(SystemCommand::ProvisionKey {}));
    let (m, _) = usb.expect(3.0, |m| {
        matches!(m, ProtocolMessage::Auth(AuthMessage::Key { .. }))
    });
    let same = matches!(&m, Some(ProtocolMessage::Auth(AuthMessage::Key { key: k, .. })) if Key::from_hex(k).as_ref() == Some(&key));
    r.check(
        "A second ProvisionKey returns the same key (stored in NVS)",
        same,
        "different key",
    );

    // Provisioning over Bluetooth is refused.
    bt.send(&sys(SystemCommand::ProvisionKey {}));
    let (m, seen) = bt.expect(2.0, |m| {
        syslog_text(m).is_some_and(|(_, t, _)| t.contains("USB"))
    });
    let leaked = seen.iter().any(|l| l.contains("\"Key\""));
    r.check(
        "ProvisionKey over Bluetooth is refused",
        m.is_some() && !leaked,
        &format!("{seen:?}"),
    );

    // Keyed, unauthenticated: read-only.
    bt.send(&set(&ids[1], 50));
    let (m, seen) = bt.expect(2.0, |m| {
        syslog_text(m)
            .is_some_and(|(_, t, p)| t.contains("not authenticated") && p == LogPriority::High)
    });
    let applied = seen.iter().any(|l| l.contains("\"level\":50"));
    r.check(
        "Keyed device: an unauthenticated command is rejected (High SysLog)",
        m.is_some() && !applied,
        &format!("{seen:?}"),
    );
    let (info, _, _) = discover(&mut bt);
    r.check(
        "Keyed device: Rediscover still works (read-only)",
        info.is_some(),
        "no System",
    );

    // Wrong key: the device is not recognized.
    let mut wrong = ClientSession::new();
    bt.send(&wrong.begin(nonce()));
    let (m, _) = bt.expect(3.0, |m| {
        matches!(m, ProtocolMessage::Auth(AuthMessage::Challenge { .. }))
    });
    if let Some(ProtocolMessage::Auth(a)) = &m {
        let reply = wrong.on_message(a, |_| Some(Key::from_bytes([0x55; 32])));
        r.check(
            "A desktop with the wrong key rejects the device's proof and sends nothing",
            reply.is_none() && matches!(wrong.status(), AuthStatus::DeviceRejected { .. }),
            &format!("{:?}", wrong.status()),
        );
    } else {
        r.check("Challenge received", false, "no challenge");
    }

    // Authenticate.
    let mut client = ClientSession::new();
    bt.send(&client.begin(nonce()));
    let (m, _) = bt.expect(3.0, |m| {
        matches!(m, ProtocolMessage::Auth(AuthMessage::Challenge { .. }))
    });
    if let Some(ProtocolMessage::Auth(a)) = &m {
        if let Some(resp) = client.on_message(a, |_| Some(key.clone())) {
            bt.send(&resp);
            let (m, _) = bt.expect(3.0, |m| {
                matches!(m, ProtocolMessage::Auth(AuthMessage::Result { .. }))
            });
            if let Some(ProtocolMessage::Auth(a)) = &m {
                client.on_message(a, |_| None);
            }
        }
    }
    r.check(
        "Mutual handshake authenticates",
        matches!(client.status(), AuthStatus::Authenticated { .. }),
        &format!("{:?}", client.status()),
    );

    let signed = client.sign(&set(&ids[1], 60)).unwrap();
    let signed_line = bt.send(&signed);
    let (m, seen) = bt.expect(2.0, |m| is_brightness(m, &ids[1], 60));
    r.check(
        "Signed SetState 60 is applied",
        m.is_some(),
        &format!("{seen:?}"),
    );

    bt.send_line(&signed_line);
    let (m, _) = bt.expect(2.0, |m| {
        syslog_text(m).is_some_and(|(_, t, _)| t.contains("replayed"))
    });
    r.check(
        "A replayed signed command is rejected",
        m.is_some(),
        "no rejection",
    );

    let mut tampered = client.sign(&set(&ids[1], 70)).unwrap();
    tampered.signed = tampered.signed.replace("70", "99");
    bt.send(&tampered);
    let (m, seen) = bt.expect(2.0, |m| {
        syslog_text(m).is_some_and(|(_, t, _)| t.contains("bad MAC"))
    });
    let applied = seen.iter().any(|l| l.contains("\"level\":99"));
    r.check(
        "A tampered signed command is rejected",
        m.is_some() && !applied,
        &format!("{seen:?}"),
    );

    bt.send(&set(&ids[1], 80));
    let (m, _) = bt.expect(2.0, |m| {
        syslog_text(m).is_some_and(|(_, t, _)| t.contains("sign"))
    });
    r.check(
        "An authenticated session still rejects plain commands",
        m.is_some(),
        "accepted",
    );

    // Colour show with signed commands.
    println!("      colour show: led1/GPIO12, led2/GPIO14, led3/GPIO27");
    let mut show_ok = true;
    for (levels, name) in [
        ([100, 0, 0], "GPIO12 only"),
        ([0, 100, 0], "GPIO14 only"),
        ([0, 0, 100], "GPIO27 only"),
        ([100, 100, 100], "all on"),
        ([20, 0, 20], "dim mix"),
        ([0, 0, 0], "off"),
    ] {
        for (id, level) in ids.iter().zip(levels) {
            let s = client.sign(&set(id, level)).unwrap();
            bt.send(&s);
            let (m, _) = bt.expect(2.0, |m| is_brightness(m, id, level));
            show_ok &= m.is_some();
        }
        println!("      {name}");
        thread::sleep(Duration::from_millis(900));
    }
    r.check(
        "Colour show: every signed command confirmed by a Brightness event",
        show_ok,
        "missing confirmation",
    );

    // Reconnect replay (2A A-V6): set a level, reconnect, rediscover.
    let s = client.sign(&set(&ids[2], 40)).unwrap();
    bt.send(&s);
    let _ = bt.expect(2.0, |m| is_brightness(m, &ids[2], 40));
    drop(bt);
    match open_retry(bt_name, 20) {
        Ok(mut bt) => {
            thread::sleep(Duration::from_millis(500));
            let (_, _, raw) = discover(&mut bt);
            let replay = raw
                .iter()
                .any(|l| l.contains(&ids[2]) && l.contains("\"level\":40"));
            r.check(
                "After reconnecting, rediscovery replays the real brightness (2A A-V6)",
                replay,
                &format!("{raw:?}"),
            );
            // A new connection starts unauthenticated.
            bt.send(&set(&ids[2], 10));
            let (m, _) = bt.expect(2.0, |m| {
                syslog_text(m).is_some_and(|(_, t, _)| t.contains("not authenticated"))
            });
            r.check(
                "A new connection is unauthenticated again",
                m.is_some(),
                "accepted",
            );
        }
        Err(e) => r.check("Bluetooth reopens", false, &e),
    }

    // Leave the device unprovisioned, as found.
    usb.send(&sys(SystemCommand::ClearKey {}));
    let (m, _) = usb.expect(3.0, |m| {
        matches!(m, ProtocolMessage::Auth(AuthMessage::KeyCleared { .. }))
    });
    r.check(
        "ClearKey over USB restores the unprovisioned state",
        m.is_some(),
        "no KeyCleared",
    );

    println!("\nRESULT {} passed, {} failed", r.pass, r.fail);
}

/// Windows can take several seconds to release an SPP port after it is closed.
fn open_retry(name: &str, secs: u64) -> Result<Port, String> {
    let deadline = Instant::now() + Duration::from_secs(secs);
    loop {
        thread::sleep(Duration::from_secs(1));
        match Port::open(name, false) {
            Ok(port) => return Ok(port),
            Err(e) if Instant::now() > deadline => return Err(e),
            Err(_) => {}
        }
    }
}

/// The key survives a reboot (NVS), then is cleared.
fn persist(usb_name: &str) {
    let mut r = Report { pass: 0, fail: 0 };
    let mut usb = Port::open(usb_name, false).unwrap();
    usb.send(&sys(SystemCommand::ProvisionKey {}));
    let (m, _) = usb.expect(3.0, |m| {
        matches!(m, ProtocolMessage::Auth(AuthMessage::Key { .. }))
    });
    let key = match &m {
        Some(ProtocolMessage::Auth(AuthMessage::Key { key, .. })) => Key::from_hex(key),
        _ => None,
    };
    r.check("Provision before reboot", key.is_some(), "no key");
    drop(usb);
    thread::sleep(Duration::from_secs(1));
    let mut usb = Port::open(usb_name, true).unwrap();
    let boot = usb.drain(6.0);
    r.check(
        "After a reset the device reports a provisioned key",
        boot.iter().any(|l| l.contains("key provisioned")),
        "no 'key provisioned' line",
    );
    usb.send(&sys(SystemCommand::ProvisionKey {}));
    let (m, _) = usb.expect(3.0, |m| {
        matches!(m, ProtocolMessage::Auth(AuthMessage::Key { .. }))
    });
    let same = matches!(&m, Some(ProtocolMessage::Auth(AuthMessage::Key { key: k, .. })) if Key::from_hex(k) == key);
    r.check(
        "The key is the same after the reboot",
        same,
        "different key",
    );
    usb.send(&sys(SystemCommand::ClearKey {}));
    let (m, _) = usb.expect(3.0, |m| {
        matches!(m, ProtocolMessage::Auth(AuthMessage::KeyCleared { .. }))
    });
    r.check("ClearKey", m.is_some(), "no KeyCleared");
    drop(usb);
    thread::sleep(Duration::from_secs(1));
    let usb = Port::open(usb_name, true).unwrap();
    let boot = usb.drain(6.0);
    r.check(
        "After clearing and a reset the device reports no key",
        boot.iter().any(|l| l.contains("no key")),
        "no 'no key' line",
    );
    println!(
        "
RESULT {} passed, {} failed",
        r.pass, r.fail
    );
}

/// Prints the lookup names of the registrations in arrival order after a reset.
fn order(usb_name: &str, bt_name: &str) {
    {
        let usb = Port::open(usb_name, true).unwrap();
        usb.drain(3.0);
    }
    let mut bt = open_retry(bt_name, 20).unwrap();
    thread::sleep(Duration::from_millis(300));
    bt.send(&sys(SystemCommand::Rediscover {}));
    let names: Vec<String> = bt
        .drain(3.0)
        .iter()
        .filter_map(
            |l| match serde_json::from_str::<ProtocolMessage>(l.trim()) {
                Ok(ProtocolMessage::Registration(r)) => Some(r.lool_up_id),
                _ => None,
            },
        )
        .collect();
    println!("ORDER {}", names.join(","));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("boot") => boot(
            &args[2],
            args.get(3).and_then(|s| s.parse().ok()).unwrap_or(10.0),
        ),
        Some("scenario") => scenario(&args[2], &args[3]),
        Some("persist") => persist(&args[2]),
        Some("order") => order(&args[2], &args[3]),
        _ => eprintln!(
            "usage: pinora-hardware-probe boot <usb> [secs] | scenario <usb> <bt> | persist <usb>"
        ),
    }
}
