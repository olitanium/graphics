pub const fn rsqrt(squared: f32) -> f32 {
    let x2 = squared * 0.5;
    let mut i = squared.to_bits();
    i = 0x5F_37_59_DF - (i >> 1);
    let mut y = f32::from_bits(i);
    y = y * (1.5 - (x2 * y * y));

    y
}
