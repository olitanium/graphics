use graphics::colour::ColourRGB;
use graphics::linear_algebra::{UnitVector, Vector};
use graphics::shader_program::ActiveShaderProgram;
use graphics::texture::Texture;

use super::super::traits::ListLightCompatible;
use crate::modelling::Cubic;

#[derive(Clone, Debug)]
pub struct SpotLight {
    pub position: Vector<3>,
    pub direction: UnitVector<3>,

    pub attenuation: [f32; 3],
    pub ambient: ColourRGB,
    pub diffuse: ColourRGB,
    pub specular: ColourRGB,

    pub cos_cut_off: f32,
    pub cos_outer_cut_off: f32,
}

impl SpotLight {
    pub(crate) fn bind_to<
        'a,
        const N: usize,
        const MAX: usize,
        L: ListLightCompatible<MAX>,
        T: Texture,
    >(
        &self,
        shader: &ActiveShaderProgram<'_, '_, 'a, (Cubic, L), T, N>,
        name: &str,
        index: usize,
    ) {
        shader.set_uniform(
            format!("{name}_vary[{index}].position"),
            self.position.homogeneous(),
        );
        shader.set_uniform(format!("{name}_vary[{index}].direction"), self.direction);

        shader.set_uniform(format!("{name}[{index}].attenuation"), self.attenuation);
        shader.set_uniform(format!("{name}[{index}].ambient"), self.ambient);
        shader.set_uniform(format!("{name}[{index}].diffuse"), self.diffuse);
        shader.set_uniform(
            format!("{name}[{index}].specular"),
            self.specular.as_array(),
        );
        shader.set_uniform(format!("{name}[{index}].cos_cut_off"), self.cos_cut_off);

        shader.set_uniform(
            format!("{name}[{index}].cos_outer_cut_off"),
            self.cos_outer_cut_off,
        );
    }
}
