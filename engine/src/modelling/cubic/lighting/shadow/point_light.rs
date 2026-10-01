use alloc::ffi::CString;
use core::iter;

use geometry::{Orientation, Pose};
use graphics::framebuffer::attachments::CubeWithDepth;
use graphics::framebuffer::traits::FramebufferWithoutExtra as _;
use graphics::framebuffer::{Builder, Framebuffer, FramebufferContext};
use graphics::linear_algebra::{Matrix, UnitVector, Vector};
use graphics::shader_program::{ActiveShaderProgram, Uniform};
use graphics::texture::{CubeMap, Texture};
use graphics::types::TexDim;
use graphics::{Draw, DrawOut, Model, ShaderProgramContext};

use crate::modelling::cubic::lighting::ShadowLightCompatible;
use crate::modelling::cubic::lighting::shadow::{GenDepthError, ShadowBindError};
use crate::modelling::cubic::lighting::simple::PointLight;
use crate::modelling::cubic::{CalculatedAnimation, camera};
use crate::modelling::{SkeletonCubic, VertexHasTangent};
use crate::opengl_shaders;

#[derive(Debug)]
pub struct ShadowPointLight {
    pub light: PointLight,
    pub framebuffer: Framebuffer<0, CubeWithDepth>,
}

impl ShadowPointLight {
    pub fn new(light: PointLight, size: TexDim) -> Self {
        let framebuffer = Builder::new_cubic()
            .cubic_depth()
            .size((size, size))
            .build();
        Self { light, framebuffer }
    }

    pub const fn set_position(&mut self, position: Vector<3>) {
        self.light.position = position;
    }

    pub const fn far_plane(&self) -> f32 {
        const FAR_PLANE: f32 = 50.0;
        FAR_PLANE
    }

    pub(crate) fn get_look_at_matrices(&self) -> [Matrix<4, 4>; 6] {
        let far_plane = self.far_plane();
        const NEAR_PLANE: f32 = 0.1;

        // perspecitve * view * model
        // (FORWARD, UP)
        [
            (UnitVector::X, -UnitVector::Y),
            (-UnitVector::X, -UnitVector::Y),
            (UnitVector::Y, UnitVector::Z),
            (-UnitVector::Y, -UnitVector::Z),
            (UnitVector::Z, -UnitVector::Y),
            (-UnitVector::Z, -UnitVector::Y),
        ]
        .map(|(forward, up)| {
            camera::builder()
                .pose(Pose::new_from_orientation_translation(
                    Orientation::new_forward_up(forward, up).expect("vector pairs different"),
                    self.light.position,
                ))
                .perspective((90.0_f32).to_radians(), 1.0, NEAR_PLANE, far_plane)
                .build()
                .look_at(&())
        })
    }

    pub(crate) fn bind_to<
        'c,
        'a,
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
            &CString::new(format!("{name}[{index}].far_plane"))?,
            &self.far_plane(),
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
            CubeMap::dyn_blank(),
        )));
    }

    pub fn gen_depth<'a>(
        &self,
        sp_context: &mut ShaderProgramContext,
        fb_context: &mut FramebufferContext,
        opaque_models: &[(&'a SkeletonCubic, CalculatedAnimation)],
        uniforms: Vec<(CString, &dyn Uniform)>,
    ) -> Result<(), GenDepthError<'a>> {
        let mut active_depth_only_shader_point =
            opengl_shaders::point_depth().use_program(sp_context);
        active_depth_only_shader_point.set_uniform_dyn(uniforms)?;

        for (index, matrix) in self.get_look_at_matrices().into_iter().enumerate() {
            active_depth_only_shader_point
                .set_uniform(&CString::new(format!("matrix[{index}]"))?, &matrix)?;
        }
        active_depth_only_shader_point.set_uniform(&c"light_position", &self.light.position)?;
        active_depth_only_shader_point.set_uniform(&c"far_plane", &self.far_plane())?;

        let mut active_light_framebuffer = self.framebuffer.bind(fb_context);

        for (model, calculated_animation) in opaque_models {
            model
                .draw(
                    &mut active_depth_only_shader_point,
                    &mut active_light_framebuffer,
                    calculated_animation,
                )
                .map_err(|error| GenDepthError::Model { error, model })?;
        }

        drop(active_depth_only_shader_point);
        Ok(())
    }
}

#[derive(Debug)]
pub struct PointLightIlluminatesGroup<'a> {
    light: &'a ShadowPointLight,
    opaque: Vec<(&'a SkeletonCubic, CalculatedAnimation)>,
}

impl<'a> PointLightIlluminatesGroup<'a> {
    pub const fn new(
        light: &'a ShadowPointLight,
        opaque: Vec<(&'a SkeletonCubic, CalculatedAnimation)>,
    ) -> Self {
        Self { light, opaque }
    }

    pub fn new_dyn(
        light: &'a ShadowPointLight,
        opaque: Vec<(&'a SkeletonCubic, CalculatedAnimation)>,
    ) -> Box<dyn Draw + 'a> {
        Box::new(Self::new(light, opaque))
    }
}

impl Draw for PointLightIlluminatesGroup<'_> {
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        self.light
            .gen_depth(sp_context, fb_context, &self.opaque, vec![])
            .unwrap();

        DrawOut::OK
    }

    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }
}
