use alloc::ffi::{CString, NulError};

use array_vec::ArrayVec;
use graphics::Model;
use graphics::framebuffer::{FramebufferBindError, FramebufferContext};
use graphics::linear_algebra::Vector;
use graphics::shader_program::{
    ActiveShaderProgram,
    ShaderProgramContext,
    ShaderSetUniformError,
    Uniform,
};
use graphics::texture::Texture;
use utils::error_boilerplate;

use super::{ShadowFarLight, ShadowPointLight, ShadowSpotLight};
use crate::modelling::cubic::lighting::shadow::ShadowBindError;
use crate::modelling::cubic::lighting::traits::ShadowLightCompatible;
use crate::modelling::cubic::{CalculatedAnimation, skeleton_cubic};
use crate::modelling::{SkeletonCubic, VertexHasTangent};

#[derive(Debug, Default)]
pub struct ShadowListLights<'a, const MAX: usize> {
    pub far: ArrayVec<&'a ShadowFarLight, MAX>,
    pub spot: ArrayVec<&'a ShadowSpotLight, MAX>,
    pub point: ArrayVec<&'a ShadowPointLight, MAX>,
}

#[derive(Debug)]
pub struct ModelGenDepthDrawError<'a> {
    pub error: skeleton_cubic::DrawError,
    pub model: &'a SkeletonCubic,
}
error_boilerplate!(<'a> ModelGenDepthDrawError<'a>);

#[derive(Debug)]
#[non_exhaustive]
pub enum GenDepthError<'a> {
    SetUniform(ShaderSetUniformError),
    NulError(NulError),
    FramebufferBind(FramebufferBindError),
    Many(Vec<ModelGenDepthDrawError<'a>>),
}
error_boilerplate!(<'a> GenDepthError<'a>);

impl From<ShaderSetUniformError> for GenDepthError<'_> {
    fn from(value: ShaderSetUniformError) -> Self {
        Self::SetUniform(value)
    }
}

impl From<NulError> for GenDepthError<'_> {
    fn from(value: NulError) -> Self {
        Self::NulError(value)
    }
}

impl<const MAX: usize> ShadowListLights<'_, MAX> {
    pub(crate) fn gen_depth<'a>(
        &self,
        sp_context: &mut ShaderProgramContext,
        fb_context: &mut FramebufferContext,
        opaque_models: &[(&'a SkeletonCubic, CalculatedAnimation)],
        target_position: Vector<3>,
        uniforms: &[(CString, &dyn Uniform)],
    ) -> Result<(), GenDepthError<'a>> {
        for light in &self.far {
            light.gen_depth(
                sp_context,
                fb_context,
                opaque_models,
                uniforms,
                target_position,
            )?;
        }

        for light in &self.spot {
            light.gen_depth(sp_context, fb_context, opaque_models, uniforms.to_vec())?;
        }

        for light in &self.point {
            light.gen_depth(sp_context, fb_context, opaque_models, uniforms.to_vec())?;
        }

        // from this point, the depth buffers should be filled with depth data.

        Ok(())
    }

    /// # Safety
    /// `active_shader` must be dropped before any changes to the light's
    /// internal framebuffers is applied.
    pub(crate) unsafe fn bind<'c, const N: usize, L, T, M>(
        &'c self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'c, M, L, T, N>,
        target: Vector<3>,
    ) -> Result<(), ShadowBindError>
    where
        L: ShadowLightCompatible<MAX>,
        T: Texture,
        M: Model,
        M::Vertex: VertexHasTangent,
    {
        const FAR_LIGHT_NAME: &str = "far";
        active_shader.set_uniform(
            &CString::new(format!("num_{FAR_LIGHT_NAME}"))?,
            &u32::try_from(self.far.len())?,
        )?;
        for (index, light) in self.far.iter().enumerate() {
            light.bind_to(active_shader, FAR_LIGHT_NAME, index, target)?;
        }
        for index in self.far.len()..MAX {
            ShadowFarLight::bind_to_ghost(active_shader, FAR_LIGHT_NAME, index)?;
        }

        const SPOT_LIGHT_NAME: &str = "spot";
        active_shader.set_uniform(
            &CString::new(format!("num_{SPOT_LIGHT_NAME}"))?,
            &u32::try_from(self.spot.len())?,
        )?;
        for (index, light) in self.spot.iter().enumerate() {
            light.bind_to(active_shader, SPOT_LIGHT_NAME, index)?;
        }

        for index in self.spot.len()..MAX {
            ShadowSpotLight::bind_to_ghost(active_shader, SPOT_LIGHT_NAME, index)?;
        }

        const POINT_LIGHT_NAME: &str = "point";
        active_shader.set_uniform(
            &CString::new(format!("num_{POINT_LIGHT_NAME}"))?,
            &u32::try_from(self.point.len())?,
        )?;
        for (index, light) in self.point.iter().enumerate() {
            light.bind_to(active_shader, POINT_LIGHT_NAME, index)?;
        }
        for index in self.point.len()..MAX {
            ShadowPointLight::bind_to_ghost(active_shader, POINT_LIGHT_NAME, index)?;
        }

        Ok(())
    }
}
