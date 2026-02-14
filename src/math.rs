//! Fixed-point math for Lissajous curves.
//!
//! Reuses the 16.16 fixed-point approach from Spirograph Engine.
//! Lissajous: x(t) = A*sin(a*t + delta), y(t) = B*sin(b*t)

/// Fixed-point scale factor: 1.0 = 65536
const FP_SHIFT: i32 = 16;
const FP_ONE: i32 = 1 << FP_SHIFT;

/// Convert integer to fixed-point.
pub fn fp(n: i32) -> i32 {
    n << FP_SHIFT
}

/// Fixed-point multiply.
pub fn fp_mul(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64) >> FP_SHIFT) as i32
}

/// Fixed-point divide.
pub fn fp_div(a: i32, b: i32) -> i32 {
    if b == 0 { return 0; }
    (((a as i64) << FP_SHIFT) / b as i64) as i32
}

/// Fast integer sine approximation.
/// Input: angle in units of 1/1024 of a full circle (0..1023).
/// Output: fixed-point sin value (-65536..65536).
pub fn sin_fp(angle: i32) -> i32 {
    let a = ((angle % 1024) + 1024) % 1024;

    let (idx, sign) = if a < 256 {
        (a, 1i32)
    } else if a < 512 {
        (512 - a, 1)
    } else if a < 768 {
        (a - 512, -1)
    } else {
        (1024 - a, -1)
    };

    // Parabolic approximation for quarter wave
    let x = idx;
    let y = (4 * x * (256 - x)) as i64;
    let raw = (y * FP_ONE as i64 / (256 * 256) as i64) as i32;

    raw * sign
}

/// Fast integer cosine.
pub fn cos_fp(angle: i32) -> i32 {
    sin_fp(angle + 256)
}

/// Lissajous x(t) = A * sin(a*t + delta)
/// Parameters: freq_a (integer ratio numerator), phase delta in 1/1024 units
/// t in 1/1024 units, amplitude A as integer pixels
pub fn lissajous_x(t: i32, freq_a: i32, delta: i32, amplitude: i32) -> i32 {
    let angle = freq_a * t + delta;
    fp_mul(fp(amplitude), sin_fp(angle))
}

/// Lissajous y(t) = B * sin(b*t)
pub fn lissajous_y(t: i32, freq_b: i32, amplitude: i32) -> i32 {
    let angle = freq_b * t;
    fp_mul(fp(amplitude), sin_fp(angle))
}

/// GCD for computing pattern period.
pub fn gcd(mut a: i32, mut b: i32) -> i32 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// LCM for computing when the curve closes.
pub fn lcm(a: i32, b: i32) -> i32 {
    if a == 0 || b == 0 { return 1; }
    (a / gcd(a, b)) * b
}
