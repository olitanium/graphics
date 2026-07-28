use core::{array, f32, iter};

use fixed_point::uf8;

use crate::{UnitVector, Vector};

pub struct CompactUnitVector<const N: usize>([uf8; N - 1])
where
    [(); N - 1]:;

impl<const N: usize> CompactUnitVector<N>
where
    [(); N - 1]:,
{
    pub fn try_normalize(vec: Vector<N, f32>) -> Option<Self> {
        let normal_vec = vec.try_normalize()?.v().into_inner();

        let mut partial_sum_sq = normal_vec.iter().scan(1.0, |state, elem| {
            *state -= elem * elem;
            let out = if *state >= 0.0 { state.sqrt() } else { 0.0 };

            Some(out)
        });

        let mut angles = array::from_fn(|index| {
            let elt = f32::atan2(partial_sum_sq.next().unwrap_or(0.0), normal_vec[index]);
            if elt.is_nan() { 0.0 } else { elt }
        });

        // Annoying hack to get the last angle to have the correct sign
        angles[N - 2] = angles[N - 2].copysign(normal_vec[N - 1]);

        let mut out_of_one = angles.map(|angle| {
            let angle = if angle < 0.0 {
                angle + f32::consts::TAU
            } else {
                angle
            };

            angle * f32::consts::FRAC_1_PI
        });

        // angels[N-2] (the last element) is out of 2 Pi, so needs to be halved here so
        out_of_one[N - 2] *= 0.5;

        let out = out_of_one.map(|float| uf8::from_f32(float).expect("less than one"));

        Some(CompactUnitVector(out))
    }

    pub fn unit(self) -> UnitVector<N, f32> {
        let mut as_float = self.0.map(|x| x.to_f32() * f32::consts::PI);
        as_float[N - 2] *= 2.0;

        let sin_partial_product = iter::once(1.0)
            .chain(as_float.iter().map(|elem| elem.sin()))
            .scan(1.0, |state, elem| {
                *state *= elem;
                Some(*state)
            });

        let coss = as_float
            .iter()
            .map(|elem| elem.cos())
            .chain(core::iter::once(1.0));

        let mut out_iter = core::iter::zip(sin_partial_product, coss).map(|(sin, cos)| sin * cos);

        let out = core::array::from_fn(|_| out_iter.next().unwrap_or(0.0));

        UnitVector::new_unchecked(out)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_many() {
        for x in -10..10 {
            for y in -10..10 {
                for z in -10..10 {
                    for s in -10..10 {
                        for t in -10..10 {
                            let input =
                                Vector::new([x as f32, y as f32, z as f32, s as f32, t as f32]);

                            let Some(normal_usual) = input.try_normalize() else {
                                continue;
                            };
                            let Some(normal_compact) = CompactUnitVector::try_normalize(input)
                            else {
                                continue;
                            };
                            let normal_roundtrip = normal_compact.unit();

                            assert_eq!(normal_usual, normal_roundtrip);
                        }
                    }
                }
            }
        }
    }
}
