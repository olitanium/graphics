use array_vec::ArrayVec;
use graphics::shader_program::ActiveShaderProgram;
use graphics::texture::Texture;

use super::super::shadow::ShadowListLights;
use super::super::traits::ListLightCompatible;
use super::{FarLight, PointLight, SpotLight};
use crate::modelling::{SimpleCubic, CubicVertex};

#[derive(Clone, Debug, Default)]
pub struct ListLights<const MAX: usize> {
    pub point: ArrayVec<PointLight, MAX>,
    pub far: ArrayVec<FarLight, MAX>,
    pub spot: ArrayVec<SpotLight, MAX>,
}

impl<const MAX: usize> ListLights<MAX> {
    pub(crate) fn bind<
        const N: usize,
        L: ListLightCompatible<MAX>,
        T: Texture,
        M,
        V: CubicVertex,
    >(
        &self,
        shader_program: &mut ActiveShaderProgram<'_, '_, '_, V, (M, L), T, N>,
    ) {
        shader_program.set_uniform("num_point", &(self.point.len() as i32));
        for (index, light) in self.point.iter().enumerate() {
            light.bind_to(shader_program, "point", index);
        }

        shader_program.set_uniform("num_spot", &(self.spot.len() as i32));
        for (index, light) in self.spot.iter().enumerate() {
            light.bind_to(shader_program, "spot", index);
        }

        shader_program.set_uniform("num_far", &(self.far.len() as i32));
        for (index, light) in self.far.iter().enumerate() {
            light.bind_to(shader_program, "far", index);
        }
    }
}

impl<const MAX: usize> From<ShadowListLights<MAX>> for ListLights<MAX> {
    fn from(_: ShadowListLights<MAX>) -> Self {
        unimplemented!()
    }
}
