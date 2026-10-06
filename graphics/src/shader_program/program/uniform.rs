use alloc::ffi::CString;
use core::cell::OnceCell;
use core::ffi::CStr;
use core::fmt::Debug;
use core::hash::Hash as _;
use std::hash::DefaultHasher;

use colour::{ColourRGB, ColourRGBA};
use linear_algebra::{Matrix, UnitVector, Vector};
use utils::error_boilerplate;

use crate::gl_call;
use crate::shader_program::GetUniformLocationError;
use crate::shader_program::program::UniformName;
use crate::shader_program::program::active_shader::IsActiveShaderProgram;
use crate::types::UniformLocation;

#[derive(Debug)]
#[non_exhaustive]
pub enum SetUniformError {
    GetLocation(GetUniformLocationError),
}
error_boilerplate!(SetUniformError);

pub trait Uniform: Debug {
    fn set_uniform(
        &self,
        name: &dyn UniformName,
        shader_program: &dyn IsActiveShaderProgram,
    ) -> Result<(), SetUniformError>;
}

macro_rules! define_uniform {
    ($typ:ty => | $self:ident, $loc:ident, $shader:ident | $out:expr) => {
        impl Uniform for $typ {
            fn set_uniform(
                &self,
                name: &dyn UniformName,
                shader_program: &dyn IsActiveShaderProgram,
            ) -> Result<(), SetUniformError> {
                match shader_program.get_uniform_location(name) {
                    Ok(location) => Ok((|$self: &Self, $loc: UniformLocation, $shader| $out)(
                        self,
                        location,
                        shader_program,
                    )),
                    Err(err) => Err(SetUniformError::GetLocation(err)),
                }
            }
        }
    };
}

macro_rules! named_define_uniform {
    ($typ:ty => | $self:ident, $name:ident, $shader:ident | $out:expr) => {
        impl Uniform for $typ {
            fn set_uniform(
                &self,
                name: &dyn UniformName,
                shader_program: &dyn IsActiveShaderProgram,
            ) -> Result<(), SetUniformError> {
                (|$self: &Self, $name, $shader| $out)(self, name, shader_program)
            }
        }
    };
}

define_uniform!(i32 => |value, location, _s| { gl_call!{ gl::Uniform1i(location.as_primitive(), *value); } } );
define_uniform!(u32 => |value, location, _s| { gl_call!{ gl::Uniform1ui(location.as_primitive(), *value); } } );
define_uniform!(f32 => |value, location, _s| { gl_call!{ gl::Uniform1f(location.as_primitive(), *value); } } );
define_uniform!(f64 => |value, location, _s| { gl_call!{ gl::Uniform1d(location.as_primitive(), *value); } } );
named_define_uniform!([i32; 1] => |value, name, s| value[0].set_uniform(name, s));
named_define_uniform!([u32; 1] => |value, name, s| value[0].set_uniform(name, s));
named_define_uniform!([f32; 1] => |value, name, s| value[0].set_uniform(name, s));
named_define_uniform!([f64; 1] => |value, name, s| value[0].set_uniform(name, s));
define_uniform!([i32; 2] => |value, location, _s| gl_call! { gl::Uniform2i(location.as_primitive(), value[0], value[1]); });
define_uniform!([u32; 2] => |value, location, _s| gl_call! { gl::Uniform2ui(location.as_primitive(), value[0], value[1]); });
define_uniform!([f32; 2] => |value, location, _s| gl_call! { gl::Uniform2f(location.as_primitive(), value[0], value[1]); });
define_uniform!([f64; 2] => |value, location, _s| gl_call! { gl::Uniform2d(location.as_primitive(), value[0], value[1]); });
define_uniform!([i32; 3] => |value, location, _s| gl_call! { gl::Uniform3i(location.as_primitive(), value[0], value[1], value[2]); });
define_uniform!([u32; 3] => |value, location, _s| gl_call! { gl::Uniform3ui(location.as_primitive(), value[0], value[1], value[2]); });
define_uniform!([f32; 3] => |value, location, _s| gl_call! { gl::Uniform3f(location.as_primitive(), value[0], value[1], value[2]); });
define_uniform!([f64; 3] => |value, location, _s| gl_call! { gl::Uniform3d(location.as_primitive(), value[0], value[1], value[2]); });
define_uniform!([i32; 4] => |value, location, _s| gl_call! { gl::Uniform4i(location.as_primitive(), value[0], value[1], value[2], value[3]); });
define_uniform!([u32; 4] => |value, location, _s| gl_call! { gl::Uniform4ui(location.as_primitive(), value[0], value[1], value[2], value[3]); });
define_uniform!([f32; 4] => |value, location, _s| gl_call! { gl::Uniform4f(location.as_primitive(), value[0], value[1], value[2], value[3]); });
define_uniform!([f64; 4] => |value, location, _s| gl_call! { gl::Uniform4d(location.as_primitive(), value[0], value[1], value[2], value[3]); });

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
define_uniform!(Matrix<$row,$col> => |value, location, _s| gl_call! { gl::$func(location.as_primitive(), 1, gl::FALSE, value.col_major().as_ptr()); });
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

struct SliceValue<'a> {
    uniform: &'a dyn UniformName,
    index: usize,
    c_string: OnceCell<CString>,
}

impl UniformName for SliceValue<'_> {
    fn as_cstr(&self) -> &CStr {
        self.c_string.get_or_init(|| {
            let name = self.uniform.as_cstr().to_string_lossy();
            let string = format!("{name}[{:0}]", self.index);
            CString::new(string).unwrap_or_default()
        })
    }

    fn hash(&self) -> DefaultHasher {
        let mut hasher = self.uniform.hash();

        self.index.hash(&mut hasher);

        hasher
    }
}

impl<U: Uniform> Uniform for [U] {
    fn set_uniform(
        &self,
        name: &dyn UniformName,
        shader_program: &dyn IsActiveShaderProgram,
    ) -> Result<(), SetUniformError> {
        for (index, value) in self.iter().enumerate() {
            let slice_value = SliceValue {
                uniform: name,
                index,
                c_string: OnceCell::new(),
            };
            value.set_uniform(&slice_value, shader_program)?;
        }

        Ok(())
    }
}

// impl<U: Uniform> Uniform for Vec<U> {
// fn set_uniform<'a>(
// &self,
// name: String,
// shader_program: &dyn IsActiveShaderProgram<'a>,
// ) -> Result<(), SetUniformError<'a>> {
// self.as_slice().set_uniform(name, shader_program)
// }
// }
