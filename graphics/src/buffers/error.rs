#[derive(Debug, Clone)]
pub enum Error {
    FrameBufferColourOutOfBounds {
        requested: usize,
        maximum: usize,
    },
    VertexArrayOutOfBounds {
        requested: usize,
        maximum: usize,
        attribute: &'static str,
    },
    VertexArrayNonHomogeneousLength {
        position: usize,
        tex_coord: usize,
        normal: Option<usize>,
        tangent: Option<usize>,
    },
}

utils::error_boilerplate!(Error);

impl From<Error> for crate::error::Error {
    fn from(value: Error) -> Self {
        Self::Buffer(value)
    }
}