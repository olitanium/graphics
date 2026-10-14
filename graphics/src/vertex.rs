use core::{gca, mem};

use linear_algebra::{UnitVector, Vector};

use crate::types::{VertexAttr, VertexAttrType};

pub trait Vertex: Copy + 'static {
    #[rustc_always_gca]
    const ELEMENT_COUNT: usize;

    #[must_use]
    fn stride() -> usize {
        mem::size_of::<Self>()
    }

    fn offsets() -> [usize; gca!(Self::ELEMENT_COUNT)];

    fn types_of() -> [VertexAttrType; gca!(Self::ELEMENT_COUNT)];
}

#[derive(Debug, Clone, Copy)]
pub struct NoVertex;

impl Vertex for NoVertex {
    const ELEMENT_COUNT: usize = gca!(0);

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        []
    }

    fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
        []
    }
}

impl<T: VertexAttr + 'static> Vertex for T {
    const ELEMENT_COUNT: usize = gca!(1);

    fn offsets() -> [usize; gca!(Self::ELEMENT_COUNT)] {
        [0; _]
    }

    fn types_of() -> [VertexAttrType; gca!(Self::ELEMENT_COUNT)] {
        [T::attr(1); _]
    }
}

macro_rules! vector {
    ($num:literal) => {
        impl Vertex for [i32; $num] {
            const ELEMENT_COUNT: usize = core::gca!(1);

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                [0]
            }

            fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
                [(i32::attr($num))]
            }
        }

        impl Vertex for [f32; $num] {
            const ELEMENT_COUNT: usize = core::gca!(1);

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                [0]
            }

            fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
                [f32::attr($num)]
            }
        }

        impl Vertex for Vector<$num> {
            const ELEMENT_COUNT: usize = core::gca!(1);

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                <[f32; $num]>::offsets()
            }

            fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
                <[f32; $num]>::types_of()
            }
        }

        impl Vertex for UnitVector<$num> {
            const ELEMENT_COUNT: usize = core::gca!(<Vector<$num> as Vertex>::ELEMENT_COUNT);

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                Vector::<$num>::offsets()
            }

            fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
                Vector::<$num>::types_of()
            }
        }
    };
}

vector!(2);
vector!(3);
vector!(4);
