use super::ActiveShaderProgram;
use linear_algebra::{Matrix, UnitVector, Vector};
use crate::texture::Texture;
use crate::types::ToPrimitive;
use crate::gl_call;
use colour::{ColourRGB, ColourRGBA};

pub trait Uniform {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    );
}

impl Uniform for i32 {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        let uniform_location = shader_program.get_uniform_location(name);
        gl_call! { gl::Uniform1i(uniform_location.to_primitive(), self); }
    }
}

impl Uniform for f32 {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        let uniform_location = shader_program.get_uniform_location(name);
        gl_call! { gl::Uniform1f(uniform_location.to_primitive(), self); }
    }
}

impl Uniform for [i32; 1] {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        self[0].set_uniform(name, shader_program)
    }
}

impl Uniform for [f32; 1] {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        self[0].set_uniform(name, shader_program)
    }
}

impl Uniform for [i32; 2] {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        let uniform_location = shader_program.get_uniform_location(name);
        gl_call! { gl::Uniform2i(uniform_location.to_primitive(), self[0], self[1]); }
    }
}

impl Uniform for [f32; 2] {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        let uniform_location = shader_program.get_uniform_location(name);
        gl_call! { gl::Uniform2f(uniform_location.to_primitive(), self[0], self[1]); }
    }
}

impl Uniform for [i32; 3] {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        let uniform_location = shader_program.get_uniform_location(name);
        gl_call! { gl::Uniform3i(uniform_location.to_primitive(), self[0], self[1], self[2]); }
    }
}

impl Uniform for [f32; 3] {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        let uniform_location = shader_program.get_uniform_location(name);
        gl_call! { gl::Uniform3f(uniform_location.to_primitive(), self[0], self[1], self[2]); }
    }
}

impl Uniform for [i32; 4] {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        let uniform_location = shader_program.get_uniform_location(name);
        gl_call! { gl::Uniform4i(uniform_location.to_primitive(), self[0], self[1], self[2], self[3]); }
    }
}

impl Uniform for [f32; 4] {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        let uniform_location = shader_program.get_uniform_location(name);
        gl_call! { gl::Uniform4f(uniform_location.to_primitive(), self[0], self[1], self[2], self[3]); }
    }
}

impl Uniform for &Matrix<4, 4> {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        let uniform_location = shader_program.get_uniform_location(name);
        gl_call! { gl::UniformMatrix4fv(
            uniform_location.to_primitive(),
            1,
            gl::FALSE,
            self.col_major().as_ptr().cast(),
        ); }
    }
}

impl Uniform for Matrix<4, 4> {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        (&self).set_uniform(name, shader_program);
    }
}

impl Uniform for Vector<1> {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        self.into_inner().set_uniform(name, shader_program);
    }
}

impl Uniform for Vector<2> {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        self.into_inner().set_uniform(name, shader_program);
    }
}

impl Uniform for Vector<3> {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        self.into_inner().set_uniform(name, shader_program);
    }
}

impl Uniform for UnitVector<3> {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        self.v().set_uniform(name, shader_program);
    }
}

impl Uniform for Vector<4> {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        self.into_inner().set_uniform(name, shader_program);
    }
}

impl Uniform for ColourRGB {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        self.as_array().set_uniform(name, shader_program);
    }
}

impl Uniform for ColourRGBA {
    fn set_uniform<M, T: Texture, const OUT: usize>(
        self,
        name: String,
        shader_program: &ActiveShaderProgram<'_, '_, '_, M, T, OUT>,
    ) {
        self.as_array().set_uniform(name, shader_program);
    }
}
