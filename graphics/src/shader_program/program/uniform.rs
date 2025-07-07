use colour::{ColourRGB, ColourRGBA};
use linear_algebra::{Matrix, UnitVector, Vector};

use super::ActiveShaderProgram;
use crate::gl_call;
use crate::shader_program::program::active_shader::IsActiveShaderProgram;
use crate::texture::Texture;
use crate::types::{UniformLocation};

pub trait Uniform {
    fn set_uniform_ref(
        &self,
        location: UniformLocation,
        shader_program: &dyn IsActiveShaderProgram,
    );

    fn set_uniform(
        self,
        location: UniformLocation,
        shader_program: &dyn IsActiveShaderProgram,
    );
}

macro_rules! define_uniform {
    ($typ:ty => |$self:ident, $loc:ident, $shader:ident| $out:expr ) => {
        impl Uniform for $typ {
            fn set_uniform_ref(&self, location: UniformLocation, shader_program: &dyn IsActiveShaderProgram) {
                //let location = location.to_primitive();
                //use $crate::types::ToPrimitive;
                ( |$self: &Self, $loc: UniformLocation, $shader| $out )(self, location, shader_program)
            }

            fn set_uniform(self, location: UniformLocation, shader_program: &dyn IsActiveShaderProgram) {
                self.set_uniform_ref(location, shader_program)
            }
        }
    };
}

define_uniform!(i32 => |value, location, _s| { gl_call!{ gl::Uniform1i(location.to_primitive(), *value) } } );
define_uniform!(f32 => |value, location, _s| { gl_call!{ gl::Uniform1f(location.to_primitive(), *value) } } );
define_uniform!([i32; 1] => |value, location, s| value[0].set_uniform_ref(location, s));
define_uniform!([f32; 1] => |value, location, s| value[0].set_uniform_ref(location, s));
define_uniform!([i32; 2] => |value, location, _s| gl_call! { gl::Uniform2i(location.to_primitive(), value[0], value[1]); });
define_uniform!([f32; 2] => |value, location, _s| gl_call! { gl::Uniform2f(location.to_primitive(), value[0], value[1]); });
define_uniform!([i32; 3] => |value, location, _s| gl_call! { gl::Uniform3i(location.to_primitive(), value[0], value[1], value[2]); });
define_uniform!([f32; 3] => |value, location, _s| gl_call! { gl::Uniform3f(location.to_primitive(), value[0], value[1], value[2]); });
define_uniform!([i32; 4] => |value, location, _s| gl_call! { gl::Uniform4i(location.to_primitive(), value[0], value[1], value[2], value[3]); });
define_uniform!([f32; 4] => |value, location, _s| gl_call! { gl::Uniform4f(location.to_primitive(), value[0], value[1], value[2], value[3]); });

macro_rules! vectors {
    ($($num:literal),*) => {
        $(
define_uniform!(Vector<$num> => |value, location, s| value.inner().set_uniform_ref(location, s));
define_uniform!(UnitVector<$num> => |value, location, s| value.v().inner().set_uniform_ref(location, s));
        )*
    };
}

vectors!(1,2,3,4);

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

define_uniform!(ColourRGB => |value, location, s| value.as_ref().set_uniform_ref(location, s));
define_uniform!(ColourRGBA => |value, location, s| value.as_ref().set_uniform_ref(location, s));