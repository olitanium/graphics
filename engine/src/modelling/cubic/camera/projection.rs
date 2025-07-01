use graphics::linear_algebra::Matrix;

#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Orthographic {
        width: f32,
        height: f32,
        near: f32,
        far: f32,
    },
    Perspective {
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
}

impl Default for Projection {
    fn default() -> Self {
        Self::Perspective {
            fov: (90.0_f32).to_radians(),
            aspect: 1.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

impl Projection {
    pub fn as_matrix(&self) -> Matrix<4, 4> {
        match self {
            &Projection::Orthographic {
                width,
                height,
                near,
                far,
            } => Matrix::transform_orthogonal(width, height, near, far),
            &Projection::Perspective {
                fov,
                aspect,
                near,
                far,
            } => Matrix::transform_perspective(fov, aspect, near, far),
        }
    }
}
