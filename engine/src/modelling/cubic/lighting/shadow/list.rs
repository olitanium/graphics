use array_vec::ArrayVec;
use graphics::Model;
use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::linear_algebra::Vector;
use graphics::shader_program::{
    ActiveShaderProgram,
    SetUniformError,
    ShaderProgramContext,
    Uniform,
};
use graphics::texture::Texture;
use utils::error_boilerplate;

use super::{ShadowFarLight, ShadowPointLight, ShadowSpotLight};
use crate::modelling::cubic::geometry::CalculatedAnimation;
use crate::modelling::cubic::lighting::shadow::ShadowBindError;
use crate::modelling::cubic::lighting::traits::ShadowLightCompatible;
use crate::modelling::cubic::simple_cubic;
use crate::modelling::{SimpleCubic, VertexHasTangent};
use crate::opengl_shaders;

#[derive(Debug, Default)]
pub struct ShadowListLights<const MAX: usize> {
    pub far: ArrayVec<ShadowFarLight, MAX>,
    pub spot: ArrayVec<ShadowSpotLight, MAX>,
    pub point: ArrayVec<ShadowPointLight, MAX>,
}

#[derive(Debug)]
pub enum GenDepthError<'a> {
    Model {
        error: simple_cubic::DrawError<'static>,
        model: &'a SimpleCubic,
    },
    SetUniform(SetUniformError<'static>),
}
error_boilerplate!(<'a> GenDepthError<'a>);

impl From<SetUniformError<'static>> for GenDepthError<'_> {
    fn from(value: SetUniformError<'static>) -> Self {
        Self::SetUniform(value)
    }
}

impl<const MAX: usize> ShadowListLights<MAX> {
    pub(crate) fn gen_depth<'a>(
        &self,
        sp_context: &mut ShaderProgramContext,
        fb_context: &mut FramebufferContext,
        complete_models: &[(&'a SimpleCubic, CalculatedAnimation)],
        target_position: Vector<3>,
        uniforms: Vec<(String, &dyn Uniform)>,
    ) -> Result<(), GenDepthError<'a>> {
        let mut active_depth_only_shader =
            opengl_shaders::far_light_depth().use_program(sp_context);
        active_depth_only_shader.set_uniform_dyn(uniforms.clone())?;

        for light in &self.far {
            // Draw the scene from the lights perspective, saving to the light's internal
            // framebuffer
            active_depth_only_shader.set_uniform(
                "projtimesview".to_string(),
                &light.projtimesview(target_position),
            )?;

            let mut active_light_framebuffer = light.framebuffer.bind(fb_context);

            for (model, calculated_animation) in complete_models {
                model
                    .draw(
                        &mut active_depth_only_shader,
                        &mut active_light_framebuffer,
                        calculated_animation,
                    )
                    .map_err(|error| GenDepthError::Model { error, model })?;
            }
        }
        drop(active_depth_only_shader);

        let mut active_depth_only_shader =
            opengl_shaders::far_light_depth().use_program(sp_context);
        active_depth_only_shader.set_uniform_dyn(uniforms.clone())?;

        for light in &self.spot {
            // Draw the scene from the lights perspective, saving to the light's internal
            // framebuffer
            active_depth_only_shader
                .set_uniform("projtimesview".to_string(), &light.projtimesview())?;

            let mut active_light_framebuffer = light.framebuffer.bind(fb_context);

            for (model, calculated_animation) in complete_models {
                model
                    .draw(
                        &mut active_depth_only_shader,
                        &mut active_light_framebuffer,
                        calculated_animation,
                    )
                    .map_err(|error| GenDepthError::Model { error, model })?;
            }
        }
        drop(active_depth_only_shader);

        let mut active_depth_only_shader_point =
            opengl_shaders::point_depth().use_program(sp_context);
        active_depth_only_shader_point.set_uniform_dyn(uniforms.clone())?;

        for light in &self.point {
            for (index, matrix) in light.get_look_at_matrices().into_iter().enumerate() {
                active_depth_only_shader_point.set_uniform(format!("matrix[{index}]"), &matrix)?;
            }
            active_depth_only_shader_point.set_uniform("light_position", &light.light.position)?;
            active_depth_only_shader_point.set_uniform("far_plane", &light.far_plane())?;

            let mut active_light_framebuffer = light.framebuffer.bind(fb_context);

            for (model, calculated_animation) in complete_models {
                model
                    .draw(
                        &mut active_depth_only_shader_point,
                        &mut active_light_framebuffer,
                        calculated_animation,
                    )
                    .map_err(|error| GenDepthError::Model { error, model })?;
            }
        }

        drop(active_depth_only_shader_point);
        // from this point, the depth buffers should be filled with depth data.

        Ok(())
    }

    /// # Safety
    /// `active_shader` must be dropped before any changes to the light's
    /// internal framebuffers is applied
    pub(crate) unsafe fn bind<
        'a,
        'c,
        const N: usize,
        L: ShadowLightCompatible<MAX>,
        T: Texture,
        M: Model,
    >(
        &'c self,
        active_shader: &mut ActiveShaderProgram<'a, '_, 'c, M, L, T, N>,
        target: Vector<3>,
    ) -> Result<(), ShadowBindError<'a>>
    where
        M::Vertex: VertexHasTangent,
    {
        const FAR_LIGHT_NAME: &str = "far";
        active_shader.set_uniform(format!("num_{FAR_LIGHT_NAME}"), &(self.far.len() as i32))?;
        for (index, light) in self.far.iter().enumerate() {
            unsafe {
                light.bind_to(active_shader, FAR_LIGHT_NAME, index, target)?;
            }
        }
        for index in self.far.len()..MAX {
            ShadowFarLight::bind_to_ghost(active_shader, FAR_LIGHT_NAME, index);
        }

        const SPOT_LIGHT_NAME: &str = "spot";
        active_shader.set_uniform(format!("num_{SPOT_LIGHT_NAME}"), &(self.spot.len() as i32))?;
        for (index, light) in self.spot.iter().enumerate() {
            unsafe {
                light.bind_to(active_shader, SPOT_LIGHT_NAME, index)?;
            }
        }

        for index in self.spot.len()..MAX {
            ShadowSpotLight::bind_to_ghost(active_shader, SPOT_LIGHT_NAME, index);
        }

        const POINT_LIGHT_NAME: &str = "point";
        active_shader.set_uniform(
            format!("num_{POINT_LIGHT_NAME}"),
            &(self.point.len() as i32),
        )?;
        for (index, light) in self.point.iter().enumerate() {
            unsafe {
                light.bind_to(active_shader, POINT_LIGHT_NAME, index)?;
            }
        }
        for index in self.point.len()..MAX {
            ShadowPointLight::bind_to_ghost(active_shader, POINT_LIGHT_NAME, index);
        }

        Ok(())
    }
}
