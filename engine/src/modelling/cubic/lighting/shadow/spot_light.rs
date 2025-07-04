use graphics::colour::ColourRGBA;
use graphics::framebuffer::attachments::WithDepth;
use graphics::framebuffer::{Builder, Framebuffer};
use graphics::linear_algebra::{Matrix, UnitVector};
use graphics::shader_program::ActiveShaderProgram;
use graphics::texture::{FlatTexture, Magnification, Texture, WrapType};
use graphics::types::TexDim;

use crate::modelling::cubic::camera::{self, Camera};
use crate::modelling::cubic::geometry::{Orientation, Pose};
use crate::modelling::cubic::lighting::simple::SpotLight;
use crate::modelling::cubic::lighting::traits::ShadowLightCompatible;
use crate::modelling::Cubic;

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
                Orientation::new_forward_up(
                    self.light.direction,
                    UnitVector::new_unchecked([0.0, 1.0, 0.0]),
                ),
                self.light.position,
            ))
            .perspective(self.light.cos_outer_cut_off, 1.0, 0.1, 20.0)
            .build()
    }

    pub(crate) unsafe fn bind_to<
        'c,
        const N: usize,
        const MAX: usize,
        L: ShadowLightCompatible<MAX>,
        T: Texture,
    >(
        &'c self,
        shader: &mut ActiveShaderProgram<'_, '_, 'c, (Cubic, L), T, N>,
        name: &str,
        index: usize,
    ) {
        self.light.bind_to(shader, name, index);
        shader.set_uniform(format!("{name}[{index}].matrix"), self.projtimesview());
        // SAFETY: Reference must not live to a point where the
        shader.register_texture(Some((format!("{name}[{index}].depth"), unsafe {
            self.framebuffer.get_attachment_ref()
        })));
    }

    pub(crate) fn bind_to_ghost<
        const N: usize,
        const MAX: usize,
        L: ShadowLightCompatible<MAX>,
        T: Texture,
    >(
        shader: &mut ActiveShaderProgram<'_, '_, '_, (Cubic, L), T, N>,
        name: &str,
        index: usize,
    ) {
        shader.register_texture(Some((
            format!("{name}[{index}].depth"),
            FlatTexture::dyn_blank(),
        )));
    }

    pub(crate) fn projtimesview(&self) -> Matrix<4, 4> {
        self.camera().look_at(())
    }
}
