//! Linear range-mapping and clamping helpers.
//!
//! The integer `range_*` functions compute `(value - in_min) * (out_max - out_min) /
//! (in_max - in_min) + out_min` in `i128`, which cannot overflow for any input of
//! these widths, and then **saturate** to the output type. Wherever the old in-type
//! arithmetic did not overflow, the results are identical: integer division truncates
//! toward zero in both. Inputs outside the input range are *not* clamped: they
//! extrapolate, then saturate at the type limits (PIN-TPL-002, Phase 1B).

/// Shared `i128` core of the integer mappings.
fn range_i128(
    value: i128,
    input_min: i128,
    input_max: i128,
    output_min: i128,
    output_max: i128,
) -> i128 {
    (value - input_min) * (output_max - output_min) / (input_max - input_min) + output_min
}

pub fn range_u32(
    value: u32,
    input_min: u32,
    input_max: u32,
    output_min: u32,
    output_max: u32,
) -> u32 {
    if input_max == input_min {
        return output_min;
    }

    range_i128(
        value.into(),
        input_min.into(),
        input_max.into(),
        output_min.into(),
        output_max.into(),
    )
    .clamp(0, i128::from(u32::MAX)) as u32
}

pub fn range_i32(
    value: i32,
    input_min: i32,
    input_max: i32,
    output_min: i32,
    output_max: i32,
) -> i32 {
    if input_max == input_min {
        return output_min;
    }

    range_i128(
        value.into(),
        input_min.into(),
        input_max.into(),
        output_min.into(),
        output_max.into(),
    )
    .clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32
}
pub fn range_f32(
    value: f32,
    input_min: f32,
    input_max: f32,
    output_min: f32,
    output_max: f32,
) -> f32 {
    if input_max == input_min {
        return output_min;
    }

    (value - input_min) * (output_max - output_min) / (input_max - input_min) + output_min
}

pub fn range_i16(
    value: i16,
    input_min: i16,
    input_max: i16,
    output_min: i16,
    output_max: i16,
) -> i16 {
    if input_max == input_min {
        return output_min;
    }

    range_i128(
        value.into(),
        input_min.into(),
        input_max.into(),
        output_min.into(),
        output_max.into(),
    )
    .clamp(i128::from(i16::MIN), i128::from(i16::MAX)) as i16
}

/// Maps a raw axis reading to `-output_max..=output_max` around its measured `centre`:
/// 0 inside the dead zone, and each side scaled separately, because the centre is
/// rarely in the middle of the raw range. The sign always follows the side of the centre.
pub fn centred_axis(raw: u16, centre: u16, deadzone: u16, raw_max: u16, output_max: i16) -> i16 {
    let (raw, centre, deadzone) = (i32::from(raw), i32::from(centre), i32::from(deadzone));
    let output_max_i32 = i32::from(output_max);
    let value = if (raw - centre).abs() <= deadzone {
        0
    } else if raw > centre {
        range_i32(
            raw,
            centre + deadzone,
            i32::from(raw_max),
            0,
            output_max_i32,
        )
    } else {
        range_i32(raw, 0, centre - deadzone, -output_max_i32, 0)
    };
    value.clamp(-output_max_i32, output_max_i32) as i16
}

pub fn constrain_f32(value: f32, min: f32, max: f32) -> f32 {
    value.clamp(min, max)
}

pub fn constrain_u32(value: u32, min: u32, max: u32) -> u32 {
    value.clamp(min, max)
}

pub fn pulse_us_to_tick(pulse_us: i32) -> u16 {
    // For PCA9685 at about 60 Hz.
    // One full PWM cycle is about 16,666 microseconds.
    // Computed in i64 so large pulse values clamp instead of overflowing.
    let tick = (i64::from(pulse_us) * 4096) / 16_666;

    tick.clamp(0, 4095) as u16
}
