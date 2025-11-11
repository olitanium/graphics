use std::fmt::Debug;

use colour::{ColourRGB, ColourRGBA};
use linear_algebra::{Matrix, UnitVector, Vector};

use crate::shader_program::program::active_shader::IsActiveShaderProgram;
use crate::types::UniformLocation;
use crate::{ApiResult, gl_call};

pub struct NameNotFoundError;

pub trait Uniform: Debug {
    fn set_uniform(
        &self,
        name: String,
        shader_program: &dyn IsActiveShaderProgram,
    ) -> ApiResult<(), !>;
}

macro_rules! define_uniform {
    ($typ:ty => | $self:ident, $loc:ident, $shader:ident | $out:expr) => {
        impl Uniform for $typ {
            fn set_uniform(
                &self,
                name: String,
                shader_program: &dyn IsActiveShaderProgram,
            ) -> ApiResult<(), !> {
                shader_program.get_uniform_location(name).map(|location| {
                    (|$self: &Self, $loc: UniformLocation, $shader| $out)(
                        self,
                        location,
                        shader_program,
                    );
                })
            }
        }
    };
}

macro_rules! named_define_uniform {
    ($typ:ty => | $self:ident, $name:ident, $shader:ident | $out:expr) => {
        impl Uniform for $typ {
            fn set_uniform(
                &self,
                name: String,
                shader_program: &dyn IsActiveShaderProgram,
            ) -> $crate::ApiResult<(), !> {
                (|$self: &Self, $name: String, $shader| $out)(self, name, shader_program)
            }
        }
    };
}

define_uniform!(i32 => |value, location, _s| { gl_call!{ gl::Uniform1i(location.to_primitive(), *value); } } );
define_uniform!(f32 => |value, location, _s| { gl_call!{ gl::Uniform1f(location.to_primitive(), *value); } } );
named_define_uniform!([i32; 1] => |value, name, s| value[0].set_uniform(name, s));
named_define_uniform!([f32; 1] => |value, name, s| value[0].set_uniform(name, s));
define_uniform!([i32; 2] => |value, location, _s| gl_call! { gl::Uniform2i(location.to_primitive(), value[0], value[1]); });
define_uniform!([f32; 2] => |value, location, _s| gl_call! { gl::Uniform2f(location.to_primitive(), value[0], value[1]); });
define_uniform!([i32; 3] => |value, location, _s| gl_call! { gl::Uniform3i(location.to_primitive(), value[0], value[1], value[2]); });
define_uniform!([f32; 3] => |value, location, _s| gl_call! { gl::Uniform3f(location.to_primitive(), value[0], value[1], value[2]); });
define_uniform!([i32; 4] => |value, location, _s| gl_call! { gl::Uniform4i(location.to_primitive(), value[0], value[1], value[2], value[3]); });
define_uniform!([f32; 4] => |value, location, _s| gl_call! { gl::Uniform4f(location.to_primitive(), value[0], value[1], value[2], value[3]); });

macro_rules! vectors {
    ($($num:literal),*) => {
        $(
named_define_uniform!(Vector<$num> => |value, name, s| value.inner().set_uniform(name, s));
named_define_uniform!(UnitVector<$num> => |value, name, s| value.v().inner().set_uniform(name, s));
        )*
    };
}

vectors!(2, 3, 4);

macro_rules! matrix {
    ($row:literal, $col:literal => $func:ident) => {
define_uniform!(Matrix<$row,$col> => |value, location, _s| gl_call! { gl::$func(location.to_primitive(), 1, gl::FALSE, value.col_major().as_ptr()); });
    };
}

matrix!(2,2 => UniformMatrix2fv);
matrix!(2,3 => UniformMatrix3x2fv);
matrix!(2,4 => UniformMatrix4x2fv);
matrix!(3,2 => UniformMatrix2x3fv);
matrix!(3,3 => UniformMatrix3fv);
matrix!(3,4 => UniformMatrix4x3fv);
matrix!(4,2 => UniformMatrix2x4fv);
matrix!(4,3 => UniformMatrix3x4fv);
matrix!(4,4 => UniformMatrix4fv);

named_define_uniform!(ColourRGB => |value, name, s| value.as_ref().set_uniform(name, s));
named_define_uniform!(ColourRGBA => |value, name, s| value.as_ref().set_uniform(name, s));

impl<U: Uniform> Uniform for [U] {
    fn set_uniform(
        &self,
        name: String,
        shader_program: &dyn IsActiveShaderProgram,
    ) -> ApiResult<(), !> {
        for (index, value) in self.into_iter().enumerate() {
            eprintln!("setting uniform: {name}[{index}] to \n{value:#?}");
            value.set_uniform(format!("{name}[{index}]"), shader_program)?;
        }

        Ok(())
    }
}

impl<U: Uniform> Uniform for Vec<U> {
    fn set_uniform(
        &self,
        name: String,
        shader_program: &dyn IsActiveShaderProgram,
    ) -> ApiResult<(), !> {
        self.as_slice().set_uniform(name, shader_program)
    }
}

impl<U: Uniform> Uniform for &U {
    fn set_uniform(
        &self,
        name: String,
        shader_program: &dyn IsActiveShaderProgram,
    ) -> ApiResult<(), !> {
        (*self).set_uniform(name, shader_program)
    }
}
