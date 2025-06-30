use std::array;
use std::cell::RefCell;
use std::rc::Rc;

use super::Quad;
use crate::buffers::vertex_array::VertexArray;
use super::QuadVertex;
use crate::builder;
use crate::texture::FlatTexture;

#[derive(Debug, Default)]
pub struct Builder<const N: usize> {
    // depth: Option<f32>,
    vertex_array: Option<VertexArray<QuadVertex>>,
    texture: Option<[Rc<RefCell<FlatTexture>>; N]>, // Option<Rc<FlatTexture>>,
}

impl<const N: usize> Builder<N> {
    // builder!(depth: Option<f32>);

    builder!(vertex_array: Option<VertexArray<QuadVertex>>);

    builder!(texture: Option<[Rc<RefCell<FlatTexture>>; N]>);
}

impl<const N: usize> Builder<N> {
    pub fn build(self) -> Quad<N> {
        Quad {
            vertex_array: self
                .vertex_array
                .unwrap_or_else(|| VertexArray::quad(-1.0, 1.0, -1.0, 1.0)),
            texture: self.texture.unwrap_or(array::from_fn(|_| {
                Rc::new(RefCell::new(FlatTexture::default()))
            })),
            // depth: self.depth.unwrap_or(0.99),
        }
    }
}
