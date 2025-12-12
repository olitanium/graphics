use graphics::colour::ColourRGB;
use graphics::linear_algebra::Vector;
use graphics::shader_program::{IsActiveShaderProgram, SetUniformError, Uniform};

#[derive(Debug)]
pub struct TestLineVertex {
    pub position: Vector<3>,
    pub colour: ColourRGB,
}

impl Uniform for TestLineVertex {
    fn set_uniform<'a>(
        &self,
        name: String,
        shader_program: &dyn IsActiveShaderProgram<'a>,
    ) -> Result<(), SetUniformError<'a>> {
        self.position
            .set_uniform(format!("{name}.position"), shader_program)?;
        self.colour
            .set_uniform(format!("{name}.colour"), shader_program)?;
        Ok(())
    }
}
