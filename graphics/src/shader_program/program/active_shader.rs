use std::collections::HashMap;
use std::ffi::CString;
use std::fmt::Debug;
use std::sync::LazyLock;
use std::{iter, ptr};

use super::uniform::Uniform;
use super::{CullFace, Error, ShaderProgram, ShaderProgramContext};
use crate::error::Result;
use crate::gl_call;
use crate::texture::{self, Texture};
use crate::types::{self, ToPrimitive, UniformLocation};

pub struct ActiveShaderProgram<'a, 'b, 'c, M, T: Texture, const OUT: usize> {
    shader_program: &'a ShaderProgram<M, OUT, T>,
    texture_list: HashMap<String, &'c dyn Texture>,
    context: &'b mut ShaderProgramContext,
}

impl<M: Debug, T: Texture + Debug, const OUT: usize> Debug
    for ActiveShaderProgram<'_, '_, '_, M, T, OUT>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveShaderProgram")
            .field("shader_program", self.shader_program)
            .field_with("texture_list", |f| {
                f.debug_list()
                    .entries(
                        self.texture_list
                            .iter()
                            .map(|(string, tex)| (string, tex.id().to_primitive())),
                    )
                    .finish()
            })
            .finish_non_exhaustive()
    }
}

impl<'a, 'b, M, T: Texture, const OUT: usize> ActiveShaderProgram<'a, 'b, '_, M, T, OUT> {
    pub fn new(
        shader_program: &'a ShaderProgram<M, OUT, T>,
        context: &'b mut ShaderProgramContext,
    ) -> Self {
        Self {
            shader_program,
            context,
            texture_list: HashMap::new(),
        }
    }
}

impl<'c, M, T: Texture, const OUT: usize> ActiveShaderProgram<'_, '_, 'c, M, T, OUT> {
    pub fn context(&self) -> &ShaderProgramContext {
        &self.context
    }

    pub fn register_texture<L: IntoIterator<Item = (String, &'c dyn Texture)>>(
        &mut self,
        texture_list: L,
    ) {
        for (string, texture) in texture_list {
            self.texture_list.insert(string, texture);
        }
    }

    pub fn validate(&self) -> Result<()> {
        gl_call! { gl::ValidateProgram(self.shader_program.id.to_primitive()); }

        let mut error = 0;
        gl_call! { gl::GetProgramiv(self.shader_program.id.to_primitive(), gl::VALIDATE_STATUS, &raw mut error); }

        if error == gl::TRUE as types::GLint {
            Ok(())
        } else {
            let mut error_length = 0;
            gl_call! { gl::GetProgramiv(self.shader_program.id.to_primitive(), gl::INFO_LOG_LENGTH, &raw mut error_length); }

            let mut error_message: Vec<i8> = iter::repeat_n(0, error_length as usize).collect(); // Vec::with_capacity(error_length as usize);

            gl_call! { gl::GetProgramInfoLog(self.shader_program.id.to_primitive(), error_length, ptr::null_mut(), error_message.as_mut_ptr()); }

            let message =
                CString::from_vec_with_nul(error_message.into_iter().map(|x| x as u8).collect())
                    .unwrap_or_else(|_| c"no useful message".into());

            Err(Error::Validate { message }.into())
        }
    }

    /// # Errors
    #[inline]
    pub fn bind_textures(&self) -> Result<()> {
        static MAX_TEX_UNITS: LazyLock<usize> = LazyLock::new(|| {
            let mut out = 0;
            gl_call! {
                gl::GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &raw mut out);
            }
            out as usize
        });

        if self.texture_list.len() <= *MAX_TEX_UNITS {
            for (index, (name, tex)) in self.texture_list.iter().enumerate() {
                tex.bind_to(index as u32);
                self.set_uniform(name.clone(), index as i32);
            }
            Ok(())
        } else {
            Err(texture::Error::BindTooHigh {
                maximum: *MAX_TEX_UNITS,
                requested: self.texture_list.len(),
            }
            .into())
        }
    }

    pub fn set_uniform<U: Uniform>(&self, name: String, value: U) {
        value.set_uniform(name, self);
    }

    pub fn get_uniform_location(&self, name: String) -> UniformLocation {
        self.shader_program.get_uniform_location(name)
    }

    pub fn cull_face(&mut self, cull_face: CullFace) -> Result<()> {
        self.context.cull_face(cull_face)
    }

    pub fn drawing_skybox(&mut self, drawing_skybox: bool) {
        self.context.drawing_skybox(drawing_skybox);
    }
}
