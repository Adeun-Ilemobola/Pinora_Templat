//! NEC infrared frame decoder (audit Phase 2C, PIN-TPL-009 / PIN-TPL-019).
//!
//! Input is a sequence of `(level_high, microseconds)` pulses as captured by the RMT
//! peripheral from a demodulating IR receiver (idle high, active low). A frame is a
//! ~9 ms low / ~4.5 ms high leader followed by 32 bits, each a ~560 µs low and a high
//! of ~560 µs (0) or ~1690 µs (1), least significant bit first: address, inverted
//! address, command, inverted command.
//!
//! Pure, so it is tested on the host. The tolerances are those of the polling decoder
//! it replaces.

/// One captured pulse.
pub type Pulse = (bool, u32);

/// Returns the command byte of the first valid NEC frame in `pulses`.
pub fn decode(pulses: &[Pulse]) -> Option<u8> {
    let start = pulses.windows(2).position(|pair| {
        let (low, high) = (pair[0], pair[1]);
        !low.0 && (8_000..=10_000).contains(&low.1) && high.0 && (3_500..=5_500).contains(&high.1)
    })? + 2;
    let bits = pulses.get(start..start + 64)?;

    let mut data = [0u8; 4];
    for (bit_index, pair) in bits.chunks_exact(2).enumerate() {
        let (low, high) = (pair[0], pair[1]);
        if low.0 || !(350..=800).contains(&low.1) || !high.0 {
            return None;
        }
        let bit = match high.1 {
            350..=900 => 0,
            1_200..=2_200 => 1,
            _ => return None,
        };
        data[bit_index / 8] |= bit << (bit_index % 8);
    }
    // Address and command are each followed by their inverse.
    (data[0] ^ data[1] == 0xFF && data[2] ^ data[3] == 0xFF).then_some(data[2])
}
