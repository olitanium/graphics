use graphics::colour::ColourRGBA;
use graphics::framebuffer::attachments::WithDepth;
use graphics::framebuffer::{Builder, Framebuffer};
use graphics::linear_algebra::{Matrix, UnitVector, Vector};
use graphics::shader_program::ActiveShaderProgram;
use graphics::texture::{FlatTexture, Magnification, Texture, WrapType};
use graphics::types::TexDim;

use crate::modelling::cubic::camera;
use crate::modelling::cubic::camera::Camera;
use crate::modelling::cubic::geometry::{Orientation, Pose};
use crate::modelling::cubic::lighting::simple::FarLight;
use crate::modelling::cubic::lighting::traits::ShadowLightCompatible;
use crate::modelling::Cubic;

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
        const FAR_PLANE: f32 = 20.0;

        let vector_from_target = self.light.direction.v().scale(FAR_PLANE / 2.0);

        let global_up = if vector_from_target[0].abs() < 0.01 && vector_from_target[2].abs() < 0.01
        {
            UnitVector::new_unchecked([1.0, 0.0, 0.0])
        } else {
            UnitVector::new_unchecked([0.0, 1.0, 0.0])
        };

        camera::builder()
            .pose(Pose::new_from_orientation_translation(
                Orientation::new_forward_up(self.light.direction, global_up),
                target - vector_from_target,
            ))
            .orthographic(10.0, 10.0, 0.1, FAR_PLANE)
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
        target: Vector<3>,
    ) {
        self.light.bind_to(shader, name, index);
        shader.set_uniform(
            format!("{name}[{index}].matrix"),
            self.projtimesview(target),
        );
        // SAFETY: Reference, therefore the ActiveShaderProgram, must not live to a
        // point where the inner framebuffer is resized.
        shader.register_texture(Some((format!("{name}[{index}].depth"), unsafe {
            self.framebuffer.get_attachment_ref()
        })));
    }

    pub(crate) fn bind_to_ghost<
        'a,
        'b,
        'c,
        const N: usize,
        const MAX: usize,
        L: ShadowLightCompatible<MAX>,
        T: Texture,
    >(
        shader: &mut ActiveShaderProgram<'a, 'b, 'c, (Cubic, L), T, N>,
        name: &str,
        index: usize,
    ) {
        shader.register_texture(Some((
            format!("{name}[{index}].depth"),
            FlatTexture::dyn_blank(),
        )));
    }

    pub(crate) fn projtimesview(&self, target: Vector<3>) -> Matrix<4, 4> {
        self.camera(target).look_at(())
    }
}
