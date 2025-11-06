use graphics::ApiResult;
use graphics::colour::ColourRGB;
use graphics::linear_algebra::Vector;
use graphics::shader_program::{IsActiveShaderProgram, NameNotFoundError, Uniform};

#[derive(Debug)]
pub struct TestLineVertex {
    pub position: Vector<3>,
    pub colour: ColourRGB,
}

impl Uniform for TestLineVertex {
    fn set_uniform(
        &self,
        name: String,
        shader_program: &dyn IsActiveShaderProgram,
    ) -> ApiResult<(), !> {
        self.position
            .set_uniform(format!("{name}.position"), shader_program)?;
        self.colour
            .set_uniform(format!("{name}.colour"), shader_program)?;
        Ok(())
    }
}
