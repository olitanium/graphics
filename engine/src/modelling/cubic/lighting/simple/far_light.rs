use super::super::traits::ListLightCompatible;
use graphics::colour::ColourRGB;
use graphics::linear_algebra::UnitVector;
use crate::modelling::Cubic;
use graphics::shader_program::ActiveShaderProgram;
use graphics::texture::Texture;

#[derive(Clone, Debug)]
pub struct FarLight {
    pub direction: UnitVector<3>,

    pub ambient: ColourRGB,
    pub diffuse: ColourRGB,
    pub specular: ColourRGB,
}

impl FarLight {
    pub(crate) fn bind_to<
        'a,
        'b,
        'c,
        const N: usize,
        const MAX: usize,
        L: ListLightCompatible<MAX>,
        T: Texture,
    >(
        &self,
        shader: &ActiveShaderProgram<'a, 'b, 'c, (Cubic, L), T, N>,
        name: &str,
        index: usize,
    ) {
        shader.set_uniform(format!("{name}_vary[{index}].direction"), self.direction);
        shader.set_uniform(format!("{name}[{index}].ambient"), self.ambient);
        shader.set_uniform(format!("{name}[{index}].diffuse"), self.diffuse);
        shader.set_uniform(format!("{name}[{index}].specular"), self.specular);
    }
}
