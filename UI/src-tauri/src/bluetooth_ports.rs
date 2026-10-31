//! Windows discovery of *outgoing* Bluetooth SPP virtual COM ports.
//!
//! `serialport` can report SPP ports as `Unknown` on Windows, so the PnP device IDs
//! are checked instead. This spawns PowerShell and blocks until it exits
//! (PIN-TPL-014).

#[cfg(windows)]
pub(crate) fn windows_bluetooth_ports() -> Result<Vec<String>, String> {
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
