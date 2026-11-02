//! Windows discovery of *outgoing* Bluetooth SPP virtual COM ports.
//!
//! `serialport` can report SPP ports as `Unknown` on Windows, so the PnP device IDs
//! are checked instead. This spawns PowerShell, which takes about 0.6 s warm and
//! several seconds cold (PIN-TPL-014). Since audit Phase 2D the commands that call it
//! run off the main thread, and the result is cached for [`CACHE_TTL`], so connecting
//! right after a refresh does not run the query again.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long a port list stays valid.
pub(crate) const CACHE_TTL: Duration = Duration::from_secs(10);

/// A time-limited cache of one value (pure, so it is unit-tested).
#[derive(Debug)]
pub(crate) struct Cache<T> {
    entry: Option<(Instant, T)>,
    ttl: Duration,
}

impl<T: Clone> Cache<T> {
    pub(crate) const fn new(ttl: Duration) -> Self {
        Self { entry: None, ttl }
    }

    /// The cached value if it is younger than the TTL, otherwise `fetch()`'s value
    /// (which is cached when it succeeds).
    pub(crate) fn get_or_fetch<E>(
        &mut self,
        now: Instant,
        fetch: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        if let Some((at, value)) = &self.entry {
            if now.duration_since(*at) < self.ttl {
                return Ok(value.clone());
            }
        }
        let value = fetch()?;
        self.entry = Some((now, value.clone()));
        Ok(value)
    }

    pub(crate) fn invalidate(&mut self) {
        self.entry = None;
    }
}

static PORTS: Mutex<Cache<Vec<String>>> = Mutex::new(Cache::new(CACHE_TTL));

/// Outgoing Bluetooth COM ports, from the cache when it is fresh. `refresh` forces a
/// new query (the port selector's Refresh button).
#[cfg(windows)]
pub(crate) fn windows_bluetooth_ports(refresh: bool) -> Result<Vec<String>, String> {
    let mut cache = PORTS.lock().map_err(|error| error.to_string())?;
    if refresh {
        cache.invalidate();
    }
    cache.get_or_fetch(Instant::now(), query_windows_bluetooth_ports)
}

#[cfg(windows)]
fn query_windows_bluetooth_ports() -> Result<Vec<String>, String> {
    // BTHENUM identifies Bluetooth serial ports; LOCALMFG&0000 is the incoming listener.
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command",
            "$ErrorActionPreference = 'Stop'; Get-CimInstance Win32_SerialPort | Where-Object { $_.PNPDeviceID -like 'BTHENUM\\*' -and $_.PNPDeviceID -notmatch '_LOCALMFG&0000\\\\' } | Select-Object -ExpandProperty DeviceID"])
        .output()
        .map_err(|error| format!("Could not inspect Windows Bluetooth COM ports: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Could not inspect Windows Bluetooth COM ports: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_value_is_reused_and_a_stale_one_refetched() {
        let mut cache = Cache::new(Duration::from_secs(10));
        let start = Instant::now();
        let mut calls = 0;
        let mut fetch = |value: u32| {
            calls += 1;
            Ok::<_, ()>(value)
        };
        assert_eq!(cache.get_or_fetch(start, || fetch(1)), Ok(1));
        assert_eq!(
            cache.get_or_fetch(start + Duration::from_secs(9), || fetch(2)),
            Ok(1)
        );
        assert_eq!(
            cache.get_or_fetch(start + Duration::from_secs(10), || fetch(3)),
            Ok(3)
        );
        cache.invalidate();
        assert_eq!(
            cache.get_or_fetch(start + Duration::from_secs(11), || fetch(4)),
            Ok(4)
        );
        assert_eq!(calls, 3);
    }

    #[test]
    fn errors_are_not_cached() {
        let mut cache: Cache<u32> = Cache::new(Duration::from_secs(10));
        let now = Instant::now();
        assert_eq!(
            cache.get_or_fetch(now, || Err::<u32, &str>("boom")),
            Err("boom")
        );
        assert_eq!(cache.get_or_fetch(now, || Ok::<u32, &str>(5)), Ok(5));
    }

    /// Times the real query (run with `--ignored --nocapture`; needs Windows).
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn measure_the_windows_query() {
        for attempt in 1..=3 {
            let started = Instant::now();
            let ports = query_windows_bluetooth_ports();
            println!("query {attempt}: {:?} -> {ports:?}", started.elapsed());
        }
        let started = Instant::now();
        let _ = windows_bluetooth_ports(false);
        let _ = windows_bluetooth_ports(false);
        println!("two cached calls: {:?}", started.elapsed());
    }
}
