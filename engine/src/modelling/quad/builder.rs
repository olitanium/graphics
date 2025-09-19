use std::array;
use std::cell::RefCell;
use std::rc::Rc;

use graphics::texture::FlatTexture;
use graphics::vertex_array::VertexArray;
use utils::{RefOwned, builder};

use super::{Quad, QuadVertex};
use crate::modelling::test_models;

#[derive(Debug, Default)]
pub struct Builder<const N: usize> {
    vertex_array: Option<VertexArray<QuadVertex>>,
    texture: Option<[Rc<RefCell<FlatTexture>>; N]>, // Option<Rc<FlatTexture>>,
}

impl<const N: usize> Builder<N> {
    builder!(vertex_array: Option<VertexArray<QuadVertex>>);

    builder!(texture: Option<[Rc<RefCell<FlatTexture>>; N]>);
}

impl<const N: usize> Builder<N> {
    pub fn build(self) -> Quad<N> {
        Quad {
            vertex_array: self.vertex_array.map_or_else(
                || RefOwned::Ref(test_models::screen_quad()),
                |arr| RefOwned::Owned(arr),
            ),
            texture: self.texture.unwrap_or(array::from_fn(|_| {
                Rc::new(RefCell::new(FlatTexture::default()))
            })),
        }
    }
}
