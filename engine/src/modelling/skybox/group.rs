use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Matrix;
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};
use graphics::{Draw, DrawOut};

use super::SkyBox;
use crate::modelling::cubic::Camera;
use crate::modelling::cubic::camera::CameraPose;

#[derive(Debug)]
pub struct Group<'a, const OUT: usize, O, D: FramebufferWithDepth<OUT>> {
    shader: &'a ShaderProgram<SkyBox, O, OUT, D::Tex>,
    framebuffer: &'a D,

    skybox: &'a SkyBox,
    look_at: Matrix<4, 4>,
    uniforms: Vec<(String, &'a dyn Uniform)>,
}

impl<'a, const OUT: usize, O, X: FramebufferWithDepth<OUT>> Group<'a, OUT, O, X> {
    pub fn new<P: CameraPose>(
        shader: &'a ShaderProgram<SkyBox, O, OUT, X::Tex>,
        framebuffer: &'a X,
        skybox: &'a SkyBox,
        camera: &Camera<P>,
        camera_hint: &P::Hint,
        uniforms: Vec<(String, &'a dyn Uniform)>,
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
            uniforms,
        })
    }
}

impl<'a, const OUT: usize, O, D: FramebufferWithDepth<OUT>> Draw for Group<'a, OUT, O, D> {
    fn draw_sized(
        self,
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> DrawOut {
        let mut active_framebuffer = self.framebuffer.bind(register);
        let mut active_shader = self.shader.use_program(marker);

        active_shader.drawing_skybox(true);
        active_shader
            .set_uniform("projtimesview".to_string(), &self.look_at)
            .unwrap();

        active_shader.set_uniform_dyn(self.uniforms).unwrap();

        self.skybox
            .draw(&mut active_shader, &mut active_framebuffer)
            .unwrap();

        active_shader.drawing_skybox(false);

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
