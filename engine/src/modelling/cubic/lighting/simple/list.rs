use array_vec::{ArrayVec, arrvec};
use graphics::Model;
use graphics::colour::ColourRGB;
use graphics::linear_algebra::Vector;
use graphics::shader_program::ActiveShaderProgram;
use graphics::texture::Texture;

use super::super::traits::ListLightCompatible;
use super::{FarLight, PointLight, SpotLight};
use crate::modelling::CubicVertex;
use crate::modelling::cubic::lighting::simple::SimpleBindError;

#[derive(Clone, Debug, Default)]
pub struct ListLights<'a, const MAX: usize> {
    pub point: ArrayVec<&'a PointLight, MAX>,
    pub far: ArrayVec<&'a FarLight, MAX>,
    pub spot: ArrayVec<&'a SpotLight, MAX>,
}

impl<const MAX: usize> ListLights<'_, MAX> {
    pub(crate) fn bind<const N: usize, L, T, M>(
        &self,
        shader_program: &mut ActiveShaderProgram<'_, '_, '_, M, L, T, N>,
    ) -> Result<(), SimpleBindError>
    where
        L: ListLightCompatible<MAX>,
        T: Texture,
        M: Model,
        M::Vertex: CubicVertex,
    {
        shader_program.set_uniform(&c"num_point", &u32::try_from(self.point.len())?)?;
        for (index, light) in self.point.iter().enumerate() {
            light.bind_to(shader_program, "point", index)?;
        }

        shader_program.set_uniform(&c"num_spot", &u32::try_from(self.spot.len())?)?;
        for (index, light) in self.spot.iter().enumerate() {
            light.bind_to(shader_program, "spot", index)?;
        }

        shader_program.set_uniform(&c"num_far", &u32::try_from(self.far.len())?)?;
        for (index, light) in self.far.iter().enumerate() {
            light.bind_to(shader_program, "far", index)?;
        }

        Ok(())
    }
}

impl<const MAX: usize> ListLights<'static, MAX> {
    pub fn new_ambient() -> Self {
        static LIGHT: PointLight = PointLight {
            position: Vector::Z,
            ambient: ColourRGB::WHITE,
            attenuation: [0.0; 3],
            diffuse: ColourRGB::BLACK,
            specular: ColourRGB::BLACK,
        };

        Self {
            point: arrvec![&LIGHT],
            ..Default::default()
        }
    }
}
