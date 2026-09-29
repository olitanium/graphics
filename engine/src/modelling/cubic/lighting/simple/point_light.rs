use alloc::ffi::CString;

use graphics::Model;
use graphics::colour::ColourRGB;
use graphics::linear_algebra::Vector;
use graphics::shader_program::ActiveShaderProgram;
use graphics::texture::Texture;

use super::super::traits::ListLightCompatible;
use crate::modelling::CubicVertex;
use crate::modelling::cubic::lighting::simple::SimpleBindError;

#[derive(Clone, Debug)]
pub struct PointLight {
    pub position: Vector<3>,

    pub attenuation: [f32; 3],

    pub ambient: ColourRGB,
    pub diffuse: ColourRGB,
    pub specular: ColourRGB,
}

impl PointLight {
    pub(crate) fn bind_to<
        'a,
        const N: usize,
        const MAX: usize,
        L: ListLightCompatible<MAX>,
        T: Texture,
        M: Model,
    >(
        &self,
        shader: &mut ActiveShaderProgram<'a, '_, '_, M, L, T, N>,
        name: &str,
        index: usize,
    ) -> Result<(), SimpleBindError<'a>>
    where
        M::Vertex: CubicVertex,
    {
        shader.set_uniform(
            &CString::new(format!("{name}_vary[{index}].position"))?,
            &self.position.homogeneous(),
        )?;
        shader.set_uniform(
            &CString::new(format!("{name}[{index}].attenuation"))?,
            &self.attenuation,
        )?;
        shader.set_uniform(
            &CString::new(format!("{name}[{index}].ambient"))?,
            &self.ambient,
        )?;
        shader.set_uniform(
            &CString::new(format!("{name}[{index}].diffuse"))?,
            &self.diffuse,
        )?;
        shader.set_uniform(
            &CString::new(format!("{name}[{index}].specular"))?,
            &self.specular,
        )?;

        Ok(())
    }
}
