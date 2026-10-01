use alloc::ffi::CString;
use core::iter;

use geometry::{Orientation, Pose};
use graphics::colour::ColourRGBA;
use graphics::framebuffer::attachments::WithDepth;
use graphics::framebuffer::traits::FramebufferWithoutExtra as _;
use graphics::framebuffer::{Builder, Framebuffer, FramebufferContext};
use graphics::linear_algebra::{Matrix, UnitVector};
use graphics::shader_program::{ActiveShaderProgram, Uniform};
use graphics::texture::{FlatTexture, Magnification, Texture, WrapType};
use graphics::types::TexDim;
use graphics::{Model, ShaderProgramContext};

use crate::modelling::cubic::CalculatedAnimation;
use crate::modelling::cubic::camera::{self, Camera};
use crate::modelling::cubic::lighting::shadow::{GenDepthError, ShadowBindError};
use crate::modelling::cubic::lighting::simple::SpotLight;
use crate::modelling::cubic::lighting::traits::ShadowLightCompatible;
use crate::modelling::{SkeletonCubic, VertexHasTangent};
use crate::opengl_shaders;

#[derive(Debug)]
pub struct ShadowSpotLight {
    pub light: SpotLight,
    pub(crate) framebuffer: Framebuffer<0, WithDepth>,
}

impl ShadowSpotLight {
    pub fn new(light: SpotLight, size: TexDim) -> Self {
        Self {
            light,
            framebuffer: Builder::new_flat()
                .depth()
                .size((size, size))
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

    pub(crate) fn camera(&self) -> Camera<Pose> {
        camera::builder()
            .pose(Pose::new_from_orientation_translation(
                Orientation::new_forward_up_try_two(
                    self.light.direction,
                    UnitVector::Y,
                    UnitVector::Z,
                )
                .expect("both versions of `up` are different, this will never fail"),
                self.light.position,
            ))
            .perspective(self.light.cos_outer_cut_off, 1.0, 0.1, 20.0)
            .build()
    }

    pub(crate) fn bind_to<
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
    ) -> Result<(), ShadowBindError<'a>>
    where
        M::Vertex: VertexHasTangent,
    {
        self.light.bind_to(shader, name, index)?;
        shader.set_uniform(
            &CString::new(format!("{name}[{index}].matrix"))?,
            &self.projtimesview(),
        )?;
        shader.register_texture(iter::once((
            CString::new(format!("{name}[{index}].depth"))?,
            self.framebuffer.get_attachment_ref(),
        )));

        Ok(())
    }

    pub(crate) fn bind_to_ghost<
        const N: usize,
        const MAX: usize,
        L: ShadowLightCompatible<MAX>,
        T: Texture,
        M: Model,
    >(
        shader: &mut ActiveShaderProgram<'_, '_, '_, M, L, T, N>,
        name: &str,
        index: usize,
    ) where
        M::Vertex: VertexHasTangent,
    {
        shader.register_texture(iter::once((
            CString::new(format!("{name}[{index}].depth")).expect("should be no Nul"),
            FlatTexture::dyn_blank(),
        )));
    }

    pub(crate) fn projtimesview(&self) -> Matrix<4, 4> {
        self.camera().look_at(&())
    }

    pub fn gen_depth<'a>(
        &self,
        sp_context: &mut ShaderProgramContext,
        fb_context: &mut FramebufferContext,
        opaque_models: &[(&'a SkeletonCubic, CalculatedAnimation)],
        uniforms: Vec<(CString, &dyn Uniform)>,
    ) -> Result<(), GenDepthError<'a>> {
        let mut active_depth_only_shader =
            opengl_shaders::farspot_light_depth().use_program(sp_context);
        active_depth_only_shader.set_uniform_dyn(uniforms)?;

        // Draw the scene from the lights perspective, saving to the light's
        // internal framebuffer
        active_depth_only_shader.set_uniform(&c"projtimesview", &self.projtimesview())?;

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
