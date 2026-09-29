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
    #[expect(
        clippy::cast_possible_truncation,
        reason = "vecs cannot be longer than i32::MAX"
    )]
    #[expect(
        clippy::cast_possible_wrap,
        reason = "vecs cannot be longer than i32::MAX"
    )]
    pub(crate) fn bind<'a, const N: usize, L: ListLightCompatible<MAX>, T: Texture, M: Model>(
        &self,
        shader_program: &mut ActiveShaderProgram<'a, '_, '_, M, L, T, N>,
    ) -> Result<(), SimpleBindError<'a>>
    where
        M::Vertex: CubicVertex,
    {
        shader_program.set_uniform(&c"num_point", &(self.point.len() as i32))?;
        for (index, light) in self.point.iter().enumerate() {
            light.bind_to(shader_program, "point", index)?;
        }

        shader_program.set_uniform(&c"num_spot", &(self.spot.len() as i32))?;
        for (index, light) in self.spot.iter().enumerate() {
            light.bind_to(shader_program, "spot", index)?;
        }

        shader_program.set_uniform(&c"num_far", &(self.far.len() as i32))?;
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
