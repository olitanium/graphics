use graphics::Draw;
use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Matrix;
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};

use super::SkyBox;
use crate::EngResult;
use crate::modelling::cubic::Camera;
use crate::modelling::cubic::camera::CameraPose;
use crate::modelling::skybox::SkyBoxVertex;

#[derive(Debug)]
pub struct Group<'a, const OUT: usize, D: FramebufferWithDepth<OUT>> {
    shader: &'a ShaderProgram<SkyBoxVertex, SkyBox, OUT, D::Tex>,
    framebuffer: &'a D,

    skybox: &'a SkyBox,
    look_at: Matrix<4, 4>,
    uniforms: Vec<(String, &'a dyn Uniform)>,
}

impl<'a, const OUT: usize, X: FramebufferWithDepth<OUT>> Group<'a, OUT, X> {
    pub fn new<O: CameraPose>(
        shader: &'a ShaderProgram<SkyBoxVertex, SkyBox, OUT, X::Tex>,
        framebuffer: &'a X,
        skybox: &'a SkyBox,
        camera: &Camera<O>,
        camera_hint: &O::Hint,
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

impl<'a, const OUT: usize, D: FramebufferWithDepth<OUT>> Draw for Group<'a, OUT, D> {
    fn draw(self: Box<Self>, register: &mut FramebufferContext, marker: &mut ShaderProgramContext) {
        let _: EngResult<(), !> = try {
            let mut active_framebuffer = self.framebuffer.bind(register);
            let mut active_shader = self.shader.use_program(marker);

            active_shader.drawing_skybox(true);
            active_shader.set_uniform("projtimesview".to_string(), &self.look_at);

            active_shader.set_uniform_dyn(self.uniforms);

            self.skybox
                .draw(&mut active_shader, &mut active_framebuffer)?;

            active_shader.drawing_skybox(false);
        };
    }
}
