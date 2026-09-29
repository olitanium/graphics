use alloc::ffi::CString;

use graphics::colour::ColourRGB;
use graphics::linear_algebra::Vector;
use graphics::shader_program::{IsActiveShaderProgram, SetUniformError, Uniform, UniformName};

#[derive(Debug)]
pub struct TestLineVertex {
    pub position: Vector<3>,
    pub colour: ColourRGB,
}

impl TestLineVertex {
    fn set_uniform<'a>(
        &self,
        name: &str,
        shader_program: &dyn IsActiveShaderProgram<'a>,
    ) -> Result<(), SetUniformError<'a>> {
        self.position.set_uniform(
            &CString::new(format!("{name}.position")).expect("should be no Nul"),
            shader_program,
        )?;
        self.colour.set_uniform(
            &CString::new(format!("{name}.colour")).expect("should be no Nul"),
            shader_program,
        )?;
        Ok(())
    }
}
