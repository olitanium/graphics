use core::error::Error;

use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::texture::{CubeMap, FlatTexture};
use graphics::{ActiveShaderProgram, Model, ShaderProgram};

use crate::geometry::Pose;
use crate::modelling::cubic::{CalculatedAnimation, SimpleCubicDrawError, SkeletonCubicDrawError};
use crate::modelling::{SimpleCubic, SkeletonCubic};
use crate::opengl_shaders;

pub trait DepthTestable {
    type Transform;
    type Model: Model + 'static;
    type Error: Error;

    fn get_shader_point() -> &'static ShaderProgram<Self::Model, (), 0, CubeMap>;

    fn get_shader_farspot() -> &'static ShaderProgram<Self::Model, (), 0, FlatTexture>;

    fn draw_depth<'a, D>(
        &'a self,
        transform: &Self::Transform,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self::Model, (), D::Tex, 0>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, 0, D>,
    ) -> Result<(), Self::Error>
    where
        D: FramebufferWithDepth<0>;
}

impl DepthTestable for SkeletonCubic {
    type Error = SkeletonCubicDrawError;
    type Model = Self;
    type Transform = CalculatedAnimation;

    fn get_shader_point() -> &'static ShaderProgram<Self::Model, (), 0, CubeMap> {
        opengl_shaders::point_depth_skeleton()
    }

    fn get_shader_farspot() -> &'static ShaderProgram<Self::Model, (), 0, FlatTexture> {
        opengl_shaders::farspot_light_depth_skeleton()
    }

    fn draw_depth<'a, D>(
        &'a self,
        transform: &Self::Transform,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self::Model, (), D::Tex, 0>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, 0, D>,
    ) -> Result<(), Self::Error>
    where
        D: FramebufferWithDepth<0>,
    {
        self.draw(active_shader, active_framebuffer, transform)
    }
}

impl DepthTestable for SimpleCubic {
    type Error = SimpleCubicDrawError;
    type Model = Self;
    type Transform = Pose;

    fn get_shader_point() -> &'static ShaderProgram<Self::Model, (), 0, CubeMap> {
        opengl_shaders::point_depth_simple()
    }

    fn get_shader_farspot() -> &'static ShaderProgram<Self::Model, (), 0, FlatTexture> {
        opengl_shaders::farspot_light_depth_simple()
    }

    fn draw_depth<'a, D>(
        &'a self,
        transform: &Self::Transform,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self::Model, (), D::Tex, 0>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, 0, D>,
    ) -> Result<(), Self::Error>
    where
        D: FramebufferWithDepth<0>,
    {
        self.draw(active_shader, active_framebuffer, *transform)
    }
}
