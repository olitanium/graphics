use std::mem;

use linear_algebra::{UnitVector, Vector};

use crate::types::{VertexAttr, VertexAttrType};

pub trait Vertex: Copy + 'static {
    type const ELEMENT_COUNT: usize;

    #[must_use]
    fn stride() -> usize {
        mem::size_of::<Self>()
    }

    fn offsets() -> [usize; Self::ELEMENT_COUNT];

    fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT];
}

#[derive(Debug, Clone, Copy)]
pub struct NoVertex;

impl Vertex for NoVertex {
    type const ELEMENT_COUNT: usize = 0;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        []
    }

    fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
        []
    }
}

impl<T: VertexAttr + 'static> Vertex for T {
    type const ELEMENT_COUNT: usize = 1;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        [0; Self::ELEMENT_COUNT]
    }

    fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
        [T::attr(1); Self::ELEMENT_COUNT]
    }
}

macro_rules! vector {
    ($num:literal) => {
        impl Vertex for [i32; $num] {
            type const ELEMENT_COUNT: usize = 1;

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                [0]
            }

            fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
                [(i32::attr($num))]
            }
        }

        impl Vertex for [f32; $num] {
            type const ELEMENT_COUNT: usize = 1;

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                [0]
            }

            fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
                [f32::attr($num)]
            }
        }

        impl Vertex for Vector<$num> {
            type const ELEMENT_COUNT: usize = 1;

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                <[f32; $num]>::offsets()
            }

            fn types_of() -> [VertexAttrType; Self::ELEMENT_COUNT] {
                <[f32; $num]>::types_of()
            }
        }

        impl Vertex for UnitVector<$num> {
            type const ELEMENT_COUNT: usize = <Vector<$num> as Vertex>::ELEMENT_COUNT;

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
