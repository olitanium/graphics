#[inline]
pub const fn rsqrt(squared: f32) -> f32 {
    let mut i = squared.to_bits();
    i = 0x5F_37_59_DF - (i >> 1);
    let mut y = f32::from_bits(i);

    let half = squared * 0.5;
    y = newton_method(half, y);
    // not normally needed, but is definitely needed by `Pose`
    y = newton_method(half, y);

    y
}

#[inline]
pub const fn newton_method(half: f32, guess: f32) -> f32 {
    guess * (half * guess).mul_add(-guess, 1.5)
}

#[cfg(test)]
mod tests {
    use crate::rsqrt;

    #[test]
    fn percent_difference() {
        let biggest = (1..1_000_000_000_u32)
            .map(|x| {
                #[expect(clippy::cast_precision_loss, reason = "test, value never too large")]
                let float = x as f32 / 1_000_000.0;
                let old_rsqrt = 1.0 / float.sqrt();
                let quake_rsqrt = rsqrt(float);

                old_rsqrt / quake_rsqrt
            })
            .max_by(f32::total_cmp);

        assert!(
            (biggest.unwrap() - 1.0).abs() < 0.01,
            "biggest deviation from sqrt is more than a percent"
        );

        // with one Newton Method
        // Some(1.0017555)
        // with two Newton Method
        // Some(1.0000048)
    }
}
