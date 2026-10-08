#[inline]
pub const fn rsqrt(squared: f32) -> f32 {
    let x2 = squared * 0.5;
    let mut i = squared.to_bits();
    i = 0x5F_37_59_DF - (i >> 1);
    let mut y = f32::from_bits(i);
    y = y * (1.5 - (x2 * y * y));
    // not normally needed, but if definitely needed by `Pose`
    y = y * (1.5 - (x2 * y * y));

    y
}

#[cfg(test)]
mod tests {
    use crate::rsqrt;

    #[test]
    fn percent_difference() {
        let biggest = (1..1_000_000_000_u32)
            .map(|x| {
                let float = x as f32 / 1_000_000.0;
                let old_rsqrt = 1.0 / float.sqrt();
                let quake_rsqrt = rsqrt(float);

                old_rsqrt / quake_rsqrt
            })
            .max_by(f32::total_cmp);

        // with one Newton Method
        // Some(1.0017555)
        // with two Newton Method
        // Some(1.0000048)
    }
}
