use graphics::{colour::ColourRGB, linear_algebra::Vector, ActiveShaderProgram};

use crate::EngResult;

pub struct TestLineVertex {
    position: Vector<3>,
    colour: ColourRGB,
}

