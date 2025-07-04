use array_vec::ArrayVec;
use graphics::framebuffer::traits::FramebufferInternals;
use graphics::framebuffer::FramebufferContext;
use graphics::linear_algebra::Vector;
use graphics::shader_program::{ActiveShaderProgram, ShaderProgramContext};
use graphics::texture::Texture;
use graphics::Result;

use super::{ShadowFarLight, ShadowPointLight, ShadowSpotLight};
use crate::modelling::cubic::lighting::traits::ShadowLightCompatible;
use crate::modelling::Cubic;
use crate::opengl_shaders;

#[derive(Debug, Default)]
pub struct ShadowListLights<const MAX: usize> {
    pub far: ArrayVec<ShadowFarLight, MAX>,
    pub spot: ArrayVec<ShadowSpotLight, MAX>,
    pub point: ArrayVec<ShadowPointLight, MAX>,
}

impl<const MAX: usize> ShadowListLights<MAX> {
    pub(crate) fn gen_depth(
        &self,
        sp_context: &mut ShaderProgramContext,
        fb_context: &mut FramebufferContext,
        complete_models: &[(&Cubic, usize /* animation */, f32 /* time */)],
        target_position: Vector<3>,
    ) -> Result<()> {
        let mut active_depth_only_shader =
            opengl_shaders::far_light_depth().use_program(sp_context);

        // let cull_face_marker =
        // active_depth_only_shader.cull_face(CullFace::FrontFace);

        for light in &self.far {
            // Draw the scene from the lights perspective, saving to the light's internal
            // framebuffer
            active_depth_only_shader.set_uniform(
                "projtimesview".to_string(),
                light.projtimesview(target_position),
            );

            let mut active_light_framebuffer = light.framebuffer.bind(fb_context);

            for (model, animation, time) in complete_models {
                model.draw(
                    &mut active_depth_only_shader,
                    &mut active_light_framebuffer,
                    *animation,
                    *time,
                )?;
            }
        }
        drop(active_depth_only_shader);

        let mut active_depth_only_shader =
            opengl_shaders::far_light_depth().use_program(sp_context);

        for light in &self.spot {
            // Draw the scene from the lights perspective, saving to the light's internal
            // framebuffer
            active_depth_only_shader
                .set_uniform("projtimesview".to_string(), light.projtimesview());

            let mut active_light_framebuffer = light.framebuffer.bind(fb_context);

            for (model, animation, time) in complete_models {
                model.draw(
                    &mut active_depth_only_shader,
                    &mut active_light_framebuffer,
                    *animation,
                    *time,
                )?;
            }
        }
        drop(active_depth_only_shader);

        let mut active_depth_only_shader_point =
            opengl_shaders::point_depth().use_program(sp_context);

        for light in &self.point {
            for (index, matrix) in light.get_look_at_matrices().into_iter().enumerate() {
                active_depth_only_shader_point.set_uniform(format!("matrix[{index}]"), matrix);
            }
            active_depth_only_shader_point
                .set_uniform("light_position".into(), light.light.position);
            active_depth_only_shader_point.set_uniform("far_plane".into(), light.far_plane());

            let mut active_light_framebuffer = light.framebuffer.bind(fb_context);

            for (model, animation, time) in complete_models {
                model.draw(
                    &mut active_depth_only_shader_point,
                    &mut active_light_framebuffer,
                    *animation,
                    *time,
                )?;
            }
        }

        drop(active_depth_only_shader_point);
        // from this point, the depth buffers should be filled with depth data.

        Ok(())
    }

    /// # Safety
    /// `active_shader` must be dropped before any changes to the light's
    /// internal framebuffers is applied
    pub(crate) unsafe fn bind<'a, const N: usize, L: ShadowLightCompatible<MAX>, T: Texture>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, (Cubic, L), T, N>,
        target: Vector<3>,
    ) {
        const FAR_LIGHT_NAME: &str = "far";
        active_shader.set_uniform(format!("num_{FAR_LIGHT_NAME}"), self.far.len() as i32);
        for (index, light) in self.far.iter().enumerate() {
            unsafe {
                light.bind_to(active_shader, FAR_LIGHT_NAME, index, target);
            }
        }
        for index in self.far.len()..MAX {
            ShadowFarLight::bind_to_ghost(active_shader, FAR_LIGHT_NAME, index);
        }

        const SPOT_LIGHT_NAME: &str = "spot";
        active_shader.set_uniform(format!("num_{SPOT_LIGHT_NAME}"), self.spot.len() as i32);
        for (index, light) in self.spot.iter().enumerate() {
            unsafe {
                light.bind_to(active_shader, SPOT_LIGHT_NAME, index);
            }
        }
        for index in self.spot.len()..MAX {
            ShadowSpotLight::bind_to_ghost(active_shader, SPOT_LIGHT_NAME, index);
        }

        const POINT_LIGHT_NAME: &str = "point";
        active_shader.set_uniform(format!("num_{POINT_LIGHT_NAME}"), self.point.len() as i32);
        for (index, light) in self.point.iter().enumerate() {
            unsafe {
                light.bind_to(active_shader, POINT_LIGHT_NAME, index);
            }
        }
        for index in self.point.len()..MAX {
            ShadowPointLight::bind_to_ghost(active_shader, POINT_LIGHT_NAME, index);
        }
    }
}
