//! Module to define a compact `UnitVector` which is embedded in one fewer
//! dimension and with smaller data types.
use core::{array, f64, iter};

use crate::{UnitVector, Vector, VectorPreferredFloat};

/// The inner type, currently hoisted as an alias for quickly swapping out.
type InnerType = fixed_point::Unitf16;

/// An in-development representation of a `UnitVector` but with one fewer
/// element and more compact form. For a 3 dimensional `UnitVector` this type
/// will use 2 dimensions.
pub struct CompactUnitVector<const N: usize>([InnerType; N - 1])
where
    [(); N - 1]:;

impl<const N: usize> CompactUnitVector<N>
where
    [(); N - 1]:,
{
    /// Normalize a `Vector<N>` and return a `CompactUnitVector<N>` if possible.
    ///
    /// Returns None if the `Vector` is a zero-vector.
    #[must_use]
    #[inline]
    pub fn try_normalize(vec: Vector<N, VectorPreferredFloat>) -> Option<Self> {
        vec.try_normalize().map(Self::from_unit_vector)
    }

    /// Convert a `UnitVector` into a `CompactUnitVector`.
    #[must_use]
    #[inline]
    pub fn from_unit_vector(uvec: UnitVector<N, VectorPreferredFloat>) -> Self {
        #[inline]
        fn inner<const N: usize>(
            unit_vec: [VectorPreferredFloat; N],
        ) -> Option<CompactUnitVector<N>>
        where
            [(); N - 1]:,
        {
            let mut partial_sum_sq = unit_vec.iter().scan(1.0, |state, elem| {
                *state -= elem * elem;
                (*state >= 0.0).then(|| state.sqrt())
            });

            let mut angles = array::from_fn(|index| {
                let elt = VectorPreferredFloat::atan2(
                    partial_sum_sq.next().unwrap_or(0.0),
                    unit_vec.get(index).copied().unwrap_or(0.0),
                );
                if elt.is_nan() { 0.0 } else { elt }
            });

            // Annoying hack to get the last angle to have the correct sign
            // angles[N - 2] = angles[N - 2].copysign(normal_vec[N - 1]);
            let ref_last = angles.last_mut()?;
            *ref_last = ref_last.copysign(*unit_vec.last()?);

            #[expect(
                clippy::allow_attributes,
                reason = "`VectorPreferredFloat` may change type"
            )]
            #[allow(
                clippy::cast_possible_truncation,
                reason = "`VectorPreferredFloat` may change type, it is preferred to start with \
                          the higher precision number"
            )]
            let mut out_of_one = angles.map(|angle| {
                let angle = if angle < 0.0 {
                    angle + const { f64::consts::TAU as VectorPreferredFloat }
                } else {
                    angle
                };

                angle * const { f64::consts::FRAC_1_PI as VectorPreferredFloat }
            });

            // angels[N-2] (the last element) is out of 2 Pi, so needs to be
            // halved here so
            *out_of_one.last_mut()? *= 0.5;

            let out = out_of_one
                .map(|val| InnerType::try_from(val).ok())
                .transpose()?;

            Some(CompactUnitVector(out))
        }

        inner(uvec.v().into_inner()).unwrap_or_else(|| Self([InnerType::default(); _]))
    }

    /// recover a `UnitVector` which has N components, from this `CompactUnit`.
    #[must_use]
    #[inline]
    pub fn unit(self) -> UnitVector<N, VectorPreferredFloat> {
        let mut as_float = self.0.map(|x| {
            VectorPreferredFloat::from(x)
                * const {
                    #[expect(
                        clippy::allow_attributes,
                        reason = "`VectorPreferredFloat` may change type"
                    )]
                    #[allow(
                        clippy::cast_possible_truncation,
                        reason = "`VectorPreferredFloat` may change type, it is preferred to \
                                  start with the higher precision number"
                    )]
                    (f64::consts::PI as VectorPreferredFloat)
                }
        });
        let Some(as_float_last) = as_float.last_mut() else {
            return UnitVector::X;
        };
        *as_float_last *= 2.0;

        let sin_partial_product = iter::once(1.0)
            .chain(as_float.iter().map(|elem| elem.sin()))
            .scan(1.0, |state, elem| {
                *state *= elem;
                Some(*state)
            });

        let coss = as_float
            .iter()
            .map(|elem| elem.cos())
            .chain(iter::once(1.0));

        let mut out_iter = iter::zip(sin_partial_product, coss).map(|(sin, cos)| sin * cos);

        let out = array::from_fn(|_| out_iter.next().unwrap_or(0.0));

        UnitVector::new_unchecked(out)
    }
}

#[cfg(test)]
mod test {
    use core::cmp;

    use super::*;

    #[test]
    fn many() {
        let mut max = 0.0;
        for x in -10..10 {
            for y in -10..10 {
                for z in -10..10 {
                    for s in -10..10 {
                        for t in -10..10_i8 {
                            let input =
                                //Vector::new([x as f32, y as f32, z as f32, s as f32, t as f32]);
                                Vector::new([x,y,z,s,t].map(VectorPreferredFloat::from));

                            let Some(normal_usual) = input.try_normalize() else {
                                continue;
                            };
                            let Some(normal_compact) = CompactUnitVector::try_normalize(input)
                            else {
                                continue;
                            };
                            let normal_roundtrip = normal_compact.unit();

                            // manual impl of Eq because the error is so high
                            // (1/255).

                            let normal_usual = normal_usual.v();
                            let normal_roundtrip = normal_roundtrip.v();

                            let diff_vec = normal_usual - normal_roundtrip;

                            for diff in diff_vec {
                                max = cmp::max_by(max, diff, VectorPreferredFloat::total_cmp);
                            }
                        }
                    }
                }
            }
        }

        // TODO: this test does nothing
    }
}
