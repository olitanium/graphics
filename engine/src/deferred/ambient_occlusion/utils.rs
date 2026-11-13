use core::num::NonZero;
use core::{array, iter};
use std::sync::LazyLock;
use std::{f32, random};

use graphics::colour::ColourRGBA;
use graphics::linear_algebra::{UnitVector, Vector, vector};
use graphics::texture::{FlatTexture, Image, TextureHasBuilder as _, WrapType};
use graphics::types::TexDim;

fn generate_0_1_uniform() -> f32 {
    let val: u32 = random::random(..);
    val as f32 * const { 1.0 / u32::MAX as f32 }
}

fn generate_random_cos_0_pi_2() -> f32 {
    let cumulative = generate_0_1_uniform();

    cumulative.asin()
}

fn generate_randon_uniform_0_to_2pi() -> f32 {
    let cumulative = generate_0_1_uniform();

    cumulative * f32::consts::TAU
}

fn generate_x2_distribution(min_bound: f32) -> f32 {
    let d = min_bound;

    let cumulative = generate_0_1_uniform();

    d / (1.0 - cumulative)
}

pub fn get_sample_vec() -> Vector<3> {
    let r = generate_x2_distribution(0.01);
    let theta = generate_random_cos_0_pi_2();
    let phi = generate_randon_uniform_0_to_2pi();

    let r_sin_theta = r * theta.sin();
    // Z needs to be the principle axis.
    vector![
        r_sin_theta * phi.cos(),
        r_sin_theta * phi.sin(),
        r * theta.cos(),
    ]
}

fn rotation_tangent_dir() -> UnitVector<3> {
    fn gen_min1_1_uniform() -> f32 {
        let cumulative = generate_0_1_uniform();
        cumulative.mul_add(2.0, -1.0)
    }

    loop {
        let trial = vector![
            gen_min1_1_uniform(),
            gen_min1_1_uniform(),
            gen_min1_1_uniform(),
        ];

        if trial.magnitude_sq() <= 1.0
            && let Some(unit) = trial.try_normalize()
        {
            return unit;
        }
    }
}

pub fn random_vec_kernel(size: (TexDim, TexDim)) -> FlatTexture {
    let pixels = size.0.get() * size.1.get();
    let pixel_iterator: Vec<_> = iter::repeat_with(rotation_tangent_dir)
        .map(|vec| ColourRGBA::new_from_arr_alpha(vec.into_inner(), 1.0))
        .take(pixels as usize)
        .collect();

    let image = Image::new_from_stream(size, pixel_iterator).unwrap();

    FlatTexture::builder()
        .image(image)
        .wrap_s_t(WrapType::Repeat)
        .build()
}

pub fn get_many_sample_vecs<const N: usize>() -> [Vector<3>; N] {
    array::from_fn(|_| get_sample_vec())
}

pub const KERNEL_SIZE: u32 = 128;
pub const KERNEL_SIZE_TEX_DIM: TexDim =
    TexDim::from_non_zero(NonZero::new(KERNEL_SIZE.checked_cast_signed().unwrap()).unwrap());
pub static RANDOM_VEC_KERNEL: LazyLock<FlatTexture> =
    LazyLock::new(|| random_vec_kernel((KERNEL_SIZE_TEX_DIM, KERNEL_SIZE_TEX_DIM)));

pub static SAMPLE_VECS_LEN: usize = 128;
pub static SAMPLE_VECS: LazyLock<[Vector<3>; SAMPLE_VECS_LEN]> =
    LazyLock::new(get_many_sample_vecs);

#[cfg(test)]
#[expect(unused, reason = "writes to files")]
mod tests {
    use core::iter;
    use std::fs;
    use std::io::Write as _;

    use graphics::linear_algebra::{UnitVector, Vector};

    use super::{
        generate_random_cos_0_pi_2,
        generate_randon_uniform_0_to_2pi,
        generate_x2_distribution,
        rotation_tangent_dir,
    };

    //#[test]
    fn get_dist_cos() {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open("cos.csv")
            .unwrap();
        writeln!(file, "data").unwrap();
        (0..10000).for_each(|_| {
            let rand = generate_random_cos_0_pi_2();
            writeln!(file, "{rand}").unwrap();
        });
    }

    //#[test]
    fn get_dist_uni() {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open("uniform.csv")
            .unwrap();
        writeln!(file, "data").unwrap();
        (0..10000).for_each(|_| {
            let rand = generate_randon_uniform_0_to_2pi();
            writeln!(file, "{rand}").unwrap();
        });
    }

    //#[test]
    fn get_dist_x2() {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open("x_2.csv")
            .unwrap();
        writeln!(file, "data").unwrap();
        (0..10000).for_each(|_| {
            let rand = generate_x2_distribution(0.1);
            writeln!(file, "{rand}").unwrap();
        });
    }

    #[test]
    fn get_many_unit() {
        let test: Vec<_> = iter::repeat_with(rotation_tangent_dir)
            .take(10_000_000)
            .collect();

        let vec: Vector<3> = test.iter().map(|x| x.v()).sum();
        let vec = vec.scale(const { 1.0 / 10_000_000.0 });

        assert!(vec.magnitude_sq() < 0.01);
    }
}
