use graphics::framebuffer::attachments::CubeWithDepth;
use graphics::framebuffer::{Builder, Framebuffer};
use graphics::linear_algebra::{Matrix, UnitVector, Vector};
use graphics::shader_program::ActiveShaderProgram;
use graphics::texture::{CubeMap, Texture};
use graphics::types::TexDim;

use crate::modelling::cubic::camera;
use crate::modelling::cubic::geometry::{Orientation, Pose};
use crate::modelling::cubic::lighting::simple::PointLight;
use crate::modelling::cubic::lighting::ShadowLightCompatible;
use crate::modelling::Cubic;

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

    pub fn set_position(&mut self, position: Vector<3>) {
        self.light.position = position;
        // self.look_at_matrices = get_look_at_matrices(&self.light);
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
        // TODO: Check that the shadows are correct for all six directions.
        [
            (
                UnitVector::new_unchecked([1.0, 0.0, 0.0]),
                UnitVector::new_unchecked([0.0, -1.0, 0.0]),
            ),
            (
                UnitVector::new_unchecked([-1.0, 0.0, 0.0]),
                UnitVector::new_unchecked([0.0, -1.0, 0.0]),
            ),
            (
                UnitVector::new_unchecked([0.0, 1.0, 0.0]),
                UnitVector::new_unchecked([0.0, 0.0, 1.0]),
            ),
            (
                UnitVector::new_unchecked([0.0, -1.0, 0.0]),
                UnitVector::new_unchecked([0.0, 0.0, -1.0]),
            ),
            (
                UnitVector::new_unchecked([0.0, 0.0, 1.0]),
                UnitVector::new_unchecked([0.0, -1.0, 0.0]),
            ),
            (
                UnitVector::new_unchecked([0.0, 0.0, -1.0]),
                UnitVector::new_unchecked([0.0, -1.0, 0.0]),
            ),
        ]
        .map(|(forward, up)| {
            camera::builder()
                .pose(Pose::new_from_orientation_translation(
                    Orientation::new_forward_up(forward, up),
                    self.light.position,
                ))
                .perspective((90.0_f32).to_radians(), 1.0, NEAR_PLANE, far_plane)
                .build()
                .look_at(())
        })
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
        shader.set_uniform(format!("{name}[{index}].far_plane"), self.far_plane());
        // SAFETY: Reference must not live to a point where RefCell::borrow_mut can be
        // called on the texture
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
            CubeMap::dyn_blank(),
        )));
    }
}
