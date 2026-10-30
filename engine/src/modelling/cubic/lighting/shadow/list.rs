use alloc::ffi::{CString, NulError};
use core::error::Error;

use array_vec::ArrayVec;
use graphics::framebuffer::{FramebufferBindError, FramebufferContext};
use graphics::linear_algebra::Vector;
use graphics::shader_program::{
    ActiveShaderProgram,
    ShaderProgramContext,
    ShaderSetUniformError,
    Uniform,
};
use graphics::texture::Texture;
use graphics::{Draw, DrawOut, Model};
use utils::error_boilerplate;

use super::{ShadowFarLight, ShadowPointLight, ShadowSpotLight};
use crate::modelling::SkeletonCubic;
use crate::modelling::cubic::CalculatedAnimation;
use crate::modelling::cubic::lighting::shadow::{BindError, DepthTestable};
use crate::modelling::cubic::lighting::traits::ShadowLightCompatible;

#[derive(Debug, Default, Clone)]
pub struct ShadowListLights<'a, const MAX: usize> {
    pub far: ArrayVec<&'a ShadowFarLight, MAX>,
    pub spot: ArrayVec<&'a ShadowSpotLight, MAX>,
    pub point: ArrayVec<&'a ShadowPointLight, MAX>,
}

#[derive(Debug)]
pub struct ModelGenDepthDrawError<'a, M: DepthTestable> {
    pub error: M::Error,
    pub model: &'a M,
}
error_boilerplate!({M: DepthTestable} <'a> ModelGenDepthDrawError<'a, M>);

#[derive(Debug)]
#[non_exhaustive]
pub enum GenDepthError<'a, M: DepthTestable> {
    SetUniform(ShaderSetUniformError),
    NulError(NulError),
    FramebufferBind(FramebufferBindError),
    Many(Vec<ModelGenDepthDrawError<'a, M>>),
}
error_boilerplate!({M: DepthTestable} <'a> GenDepthError<'a, M>);

impl<M: DepthTestable> From<ShaderSetUniformError> for GenDepthError<'_, M> {
    fn from(value: ShaderSetUniformError) -> Self {
        Self::SetUniform(value)
    }
}

impl<M: DepthTestable> From<NulError> for GenDepthError<'_, M> {
    fn from(value: NulError) -> Self {
        Self::NulError(value)
    }
}

impl<const MAX: usize> ShadowListLights<'_, MAX> {
    pub(crate) fn gen_depth<'a, M>(
        &self,
        sp_context: &mut ShaderProgramContext,
        fb_context: &mut FramebufferContext,
        opaque_models: &[(&'a M, M::Transform)],
        target_position: Vector<3>,
        uniforms: &[(CString, &dyn Uniform)],
    ) -> Result<(), GenDepthError<'a, M>>
    where
        M: DepthTestable,
    {
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

    pub(crate) fn bind<'c, const N: usize, L, T, M>(
        &'c self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'c, M, L, T, N>,
        target: Vector<3>,
    ) -> Result<(), BindError>
    where
        L: ShadowLightCompatible<MAX>,
        T: Texture,
        M: Model,
        // M::Vertex: VertexHasTangent,
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

#[derive(Debug)]
pub struct ListLightIlluminatesGroup<'a, M: DepthTestable, const MAX: usize> {
    light: ShadowListLights<'a, MAX>,
    opaque: Vec<(&'a M, M::Transform)>,
    target: Vector<3>,
}

impl<'a, M: DepthTestable, const MAX: usize> ListLightIlluminatesGroup<'a, M, MAX> {
    pub const fn new(
        light: ShadowListLights<'a, MAX>,
        opaque: Vec<(&'a M, M::Transform)>,
        target: Vector<3>,
    ) -> Self {
        Self {
            light,
            opaque,
            target,
        }
    }

    pub fn new_dyn(
        light: ShadowListLights<'a, MAX>,
        opaque: Vec<(&'a M, M::Transform)>,
        target: Vector<3>,
    ) -> Box<dyn Draw + 'a> {
        Box::new(Self::new(light, opaque, target))
    }
}

impl<M: DepthTestable, const MAX: usize> Draw for ListLightIlluminatesGroup<'_, M, MAX> {
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let _ = self
            .light
            .gen_depth(sp_context, fb_context, &self.opaque, self.target, &[]);
        //.map_err(|err| err.to_string())?;

        DrawOut::OK
    }

    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }
}
