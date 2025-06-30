use super::SkyBox;
use crate::buffers::framebuffer::{FramebufferContext, FramebufferWithDepth};
use crate::linear_algebra::Matrix;
use crate::modelling::cubic::geometry::YieldsPose;
use crate::modelling::cubic::Camera;
use crate::modelling::draw::Draw;
use crate::shader_program::{ShaderProgram, ShaderProgramContext};
use crate::Result;

#[derive(Debug)]
pub struct Group<'a, const OUT: usize, D: FramebufferWithDepth<OUT>> {
    shader: &'a ShaderProgram<SkyBox, OUT, D::Tex>,
    framebuffer: &'a D,

    skybox: &'a SkyBox,
    look_at: Matrix<4, 4>,
}

impl<'a, const OUT: usize, X: FramebufferWithDepth<OUT>> Group<'a, OUT, X> {
    pub fn new<O: YieldsPose>(
        shader: &'a ShaderProgram<SkyBox, OUT, X::Tex>,
        framebuffer: &'a X,
        skybox: &'a SkyBox,
        camera: &Camera<O>,
        camera_hint: O::Hint,
    ) -> Box<Self> {
        let proj = camera.projection;
        let view = {
            let old_view: Matrix<3, 3> = camera.view(camera_hint).truncate();
            let mut view: Matrix<4, 4> = old_view.truncate();
            view[(3, 3)] = 1.0;
            view
        };

        let look_at = proj.as_matrix() * view;

        Box::new(Self {
            shader,
            framebuffer,
            skybox,
            look_at,
        })
    }
}

impl<'a, const OUT: usize, D: FramebufferWithDepth<OUT>> Draw for Group<'a, OUT, D> {
    fn draw(
        self: Box<Self>,
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> Result<()> {
        let mut active_framebuffer = self.framebuffer.bind(register);
        let mut active_shader = self.shader.use_program(marker);

        active_shader.drawing_skybox(true);
        active_shader.set_uniform("projtimesview".to_string(), &self.look_at);

        self.skybox
            .draw(&mut active_shader, &mut active_framebuffer)?;

        active_shader.drawing_skybox(false);

        Ok(())
    }
}
