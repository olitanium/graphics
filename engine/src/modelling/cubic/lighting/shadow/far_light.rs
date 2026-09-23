use core::iter;

use geometry::{Orientation, Pose};
use graphics::colour::ColourRGBA;
use graphics::framebuffer::attachments::WithDepth;
use graphics::framebuffer::traits::FramebufferWithoutExtra as _;
use graphics::framebuffer::{Builder, Framebuffer, FramebufferContext};
use graphics::linear_algebra::{Matrix, UnitVector, Vector};
use graphics::shader_program::{ActiveShaderProgram, Uniform};
use graphics::texture::{FlatTexture, Magnification, Texture, WrapType};
use graphics::types::TexDim;
use graphics::{Model, ShaderProgramContext};

use crate::modelling::cubic::camera::Camera;
use crate::modelling::cubic::lighting::shadow::{GenDepthError, ShadowBindError};
use crate::modelling::cubic::lighting::simple::FarLight;
use crate::modelling::cubic::lighting::traits::ShadowLightCompatible;
use crate::modelling::cubic::{CalculatedAnimation, camera};
use crate::modelling::{SkeletonCubic, VertexHasTangent};
use crate::opengl_shaders;

#[derive(Debug)]
pub struct ShadowFarLight {
    pub light: FarLight,
    pub(crate) framebuffer: Framebuffer<0, WithDepth>,
}

impl ShadowFarLight {
    pub fn new(light: FarLight, size: (TexDim, TexDim)) -> Self {
        Self {
            light,
            framebuffer: Builder::new_flat()
                .depth()
                .size(size)
                .map_attachment(|tex_builder| {
                    tex_builder
                        .wrap_s_t(WrapType::ClampToBorder(ColourRGBA::new([
                            0.0, 0.0, 0.0, 1.0,
                        ])))
                        .mag_filter(Magnification::Linear)
                })
                .build(),
        }
    }

    pub(crate) fn camera(&self, target: Vector<3>) -> Camera<Pose> {
        const NEAR_PLANE: f32 = 0.1;
        const FAR_PLANE: f32 = 20.0;
        const LIGHT_PLANE_WIDTH: f32 = 10.0;

        let vector_from_target = self.light.direction.v().scale(FAR_PLANE / 2.0);

        // let [x, _, z] = vector_from_target.into_inner();
        // let global_up = if x.abs() < 0.01 && z.abs() < 0.01 {
        // UnitVector::X
        // } else {
        // UnitVector::Y
        // };

        // TODO: (TEST)

        camera::builder()
            .pose(Pose::new_from_orientation_translation(
                Orientation::new_forward_up_try_two(
                    self.light.direction,
                    UnitVector::X,
                    UnitVector::Z,
                )
                .expect("both up variables are different so this is guaranteed to succeed."),
                target - vector_from_target,
            ))
            .orthographic(LIGHT_PLANE_WIDTH, LIGHT_PLANE_WIDTH, NEAR_PLANE, FAR_PLANE)
            .build()
    }

    pub(crate) unsafe fn bind_to<
        'a,
        'c,
        const N: usize,
        const MAX: usize,
        L: ShadowLightCompatible<MAX>,
        T: Texture,
        M: Model,
    >(
        &'c self,
        shader: &mut ActiveShaderProgram<'a, '_, 'c, M, L, T, N>,
        name: &str,
        index: usize,
        target: Vector<3>,
    ) -> Result<(), ShadowBindError<'a>>
    where
        M::Vertex: VertexHasTangent,
    {
        self.light.bind_to(shader, name, index)?;
        shader.set_uniform(
            format!("{name}[{index}].matrix"),
            &self.projtimesview(target),
        )?;

        shader.register_texture(iter::once((
            format!("{name}[{index}].depth"),
            self.framebuffer.get_attachment_ref(),
        )));

        Ok(())
    }

    pub(crate) fn bind_to_ghost<
        'a,
        'b,
        'c,
        const N: usize,
        const MAX: usize,
        L: ShadowLightCompatible<MAX>,
        T: Texture,
        M: Model,
    >(
        shader: &mut ActiveShaderProgram<'a, 'b, 'c, M, L, T, N>,
        name: &str,
        index: usize,
    ) where
        M::Vertex: VertexHasTangent,
    {
        shader.register_texture(iter::once((
            format!("{name}[{index}].depth"),
            FlatTexture::dyn_blank(),
        )));
    }

    pub(crate) fn projtimesview(&self, target: Vector<3>) -> Matrix<4, 4> {
        self.camera(target).look_at(&())
    }

    pub fn gen_depth<'a>(
        &self,
        sp_context: &mut ShaderProgramContext,
        fb_context: &mut FramebufferContext,
        opaque_models: &[(&'a SkeletonCubic, CalculatedAnimation)],
        uniforms: Vec<(String, &dyn Uniform)>,
        target: Vector<3>,
    ) -> Result<(), GenDepthError<'a>> {
        let mut active_depth_only_shader =
            opengl_shaders::farspot_light_depth().use_program(sp_context);
        active_depth_only_shader.set_uniform_dyn(uniforms.to_vec())?;

        // Draw the scene from the lights perspective, saving to the light's
        // internal framebuffer
        active_depth_only_shader
            .set_uniform("projtimesview".to_string(), &self.projtimesview(target))?;

        let mut active_light_framebuffer = self.framebuffer.bind(fb_context);

        for (model, calculated_animation) in opaque_models {
            model
                .draw(
                    &mut active_depth_only_shader,
                    &mut active_light_framebuffer,
                    calculated_animation,
                )
                .map_err(|error| GenDepthError::Model { error, model })?;
        }

        drop(active_depth_only_shader);

        Ok(())
    }
}
