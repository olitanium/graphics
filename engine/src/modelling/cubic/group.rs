use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::shader_program::{ShaderProgram, ShaderProgramContext};
use graphics::texture::FlatTexture;
use graphics::{Draw, Result};

use super::Camera;
use super::geometry::YieldsPose;
use super::lighting::simple::ListLights;
use super::model::Cubic;

#[derive(Debug)]
pub struct Group<'a, const MAX: usize, const OUT: usize, D: FramebufferWithDepth<OUT>> {
    shader: &'a ShaderProgram<(Cubic, ListLights<MAX>), OUT, D::Tex>,
    framebuffer: &'a D,

    camera_pos: Vector<3>,
    camera_look_at: Matrix<4, 4>,

    lights: &'a ListLights<MAX>,
    opaque: Vec<(&'a Cubic, usize /* animation */, f32 /* time */)>,
}

impl<'a, const MAX: usize, const OUT: usize, D: FramebufferWithDepth<OUT, Tex = FlatTexture>>
    Group<'a, MAX, OUT, D>
{
    pub fn new<O: YieldsPose>(
        shader: &'a ShaderProgram<(Cubic, ListLights<MAX>), OUT, FlatTexture>,
        framebuffer: &'a D,

        camera: &Camera<O>,
        hint: O::Hint,
        lights: &'a ListLights<MAX>,
        opaque: Vec<(&'a Cubic, usize /* animation */, f32 /* time */)>,
    ) -> Box<Self>
    where
        O::Hint: Clone,
    {
        Box::new(Self {
            shader,
            framebuffer,
            camera_pos: camera.position(hint.clone()),
            camera_look_at: camera.look_at(hint),
            lights,
            opaque,
        })
    }
}

impl<'a, const MAX: usize, const OUT: usize, D: FramebufferWithDepth<OUT>> Draw
    for Group<'a, MAX, OUT, D>
{
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<()> {
        let mut active_shader = self.shader.use_program(sp_context);
        let mut active_framebuffer = self.framebuffer.bind(fb_context);

        self.lights.bind(&active_shader);
        let camera_pos = self.camera_pos;

        active_shader.set_uniform("projtimesview".to_string(), self.camera_look_at);
        active_shader.set_uniform("camera_postion".to_string(), camera_pos.homogeneous());

        for (model, animation, time) in self.opaque {
            model.draw(&mut active_shader, &mut active_framebuffer, animation, time)?;
        }

        Ok(())
    }
}
