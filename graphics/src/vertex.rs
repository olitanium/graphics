use std::mem;

use linear_algebra::{UnitVector, Vector};

use crate::types::{VertexAttr, VertexAttrType};

pub trait Vertex: Copy {
    const ELEMENT_COUNT: usize;

    fn stride() -> usize {
        mem::size_of::<Self>()
    }

    fn offsets() -> [usize; Self::ELEMENT_COUNT];

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT];
}

#[derive(Debug, Clone, Copy)]
pub struct NoVertex;

impl Vertex for NoVertex {
    const ELEMENT_COUNT: usize = 0;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        []
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        []
    }
}

impl<T: VertexAttr> Vertex for T {
    const ELEMENT_COUNT: usize = 1;

    fn offsets() -> [usize; Self::ELEMENT_COUNT] {
        [0; Self::ELEMENT_COUNT]
    }

    fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
        [(T::ATTR, 1); Self::ELEMENT_COUNT]
    }
}

macro_rules! vector {
    ($num:literal) => {
        impl Vertex for [i32; $num] {
            const ELEMENT_COUNT: usize = 1;

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                [0]
            }

            fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
                [(i32::ATTR, $num)]
            }
        }

        impl Vertex for [f32; $num] {
            const ELEMENT_COUNT: usize = 1;

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                [0]
            }

            fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
                [(f32::ATTR, $num)]
            }
        }

        impl Vertex for Vector<$num> {
            const ELEMENT_COUNT: usize = 1;

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                <[f32; $num]>::offsets()
            }

            fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
                <[f32; $num]>::types_of()
            }
        }

        impl Vertex for UnitVector<$num> {
            const ELEMENT_COUNT: usize = Vector::<$num>::ELEMENT_COUNT;

            fn offsets() -> [usize; Self::ELEMENT_COUNT] {
                Vector::<$num>::offsets()
            }

            fn types_of() -> [(VertexAttrType, usize); Self::ELEMENT_COUNT] {
                Vector::<$num>::types_of()
            }
        }
    };
}

vector!(2);
vector!(3);
vector!(4);
