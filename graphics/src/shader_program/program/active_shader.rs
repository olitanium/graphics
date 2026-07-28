use std::collections::HashMap;
use std::ffi::{CString, NulError};
use std::fmt::Debug;
use std::sync::LazyLock;
use std::{iter, ptr};

use utils::error_boilerplate;

use super::uniform::Uniform;
use super::{CullFace, ShaderProgram, ShaderProgramContext};
use crate::shader_program::program::context::FaceCullingForcedByProgramError;
use crate::shader_program::program::uniform::SetUniformError;
use crate::shader_program::program::{GetUniformLocationError, uniform};
use crate::texture::Texture;
use crate::types::{self, UniformLocation};
use crate::{Model, gl_call};

pub struct ActiveShaderProgram<'a, 'b, 'c, M: Model, O, T: Texture, const OUT: usize> {
    shader_program: &'a ShaderProgram<M, O, OUT, T>,
    context: &'b mut ShaderProgramContext,
    texture_list: HashMap<String, &'c dyn Texture>,
}

impl<M: Model + Debug, O, T: Texture + Debug, const OUT: usize> Debug
    for ActiveShaderProgram<'_, '_, '_, M, O, T, OUT>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveShaderProgram")
            .field("shader_program", self.shader_program)
            .field_with("texture_list", |f| {
                f.debug_list()
                    .entries(
                        self.texture_list
                            .iter()
                            .map(|(string, tex)| (string, tex.id().as_primitive())),
                    )
                    .finish()
            })
            .finish_non_exhaustive()
    }
}

impl<'a, 'b, M: Model, O, T: Texture, const OUT: usize>
    ActiveShaderProgram<'a, 'b, '_, M, O, T, OUT>
{
    pub fn new(
        shader_program: &'a ShaderProgram<M, O, OUT, T>,
        context: &'b mut ShaderProgramContext,
    ) -> Self {
        Self {
            shader_program,
            context,
            texture_list: HashMap::new(),
        }
    }
}

impl<'c, M: Model, O, T: Texture, const OUT: usize> ActiveShaderProgram<'_, '_, 'c, M, O, T, OUT> {
    pub fn context(&self) -> &ShaderProgramContext {
        self.context
    }

    pub fn register_texture<L: IntoIterator<Item = (String, &'c dyn Texture)>>(
        &mut self,
        texture_list: L,
    ) {
        for (string, texture) in texture_list {
            self.texture_list.insert(string, texture);
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ValidateError {
    Message(CString),
}
error_boilerplate!(ValidateError);

impl<'c, M: Model, O, T: Texture, const OUT: usize> ActiveShaderProgram<'_, '_, 'c, M, O, T, OUT> {
    pub fn validate(&self) -> Result<(), ValidateError> {
        gl_call! { gl::ValidateProgram(self.shader_program.id.as_primitive()); }

        let mut error = 0;
        gl_call! { gl::GetProgramiv(self.shader_program.id.as_primitive(), gl::VALIDATE_STATUS, &raw mut error); }

        if error == gl::TRUE as types::GLint {
            Ok(())
        } else {
            let mut error_length = 0;
            gl_call! { gl::GetProgramiv(self.shader_program.id.as_primitive(), gl::INFO_LOG_LENGTH, &raw mut error_length); }

            let mut error_message: Vec<i8> = iter::repeat_n(0, error_length as usize).collect(); // Vec::with_capacity(error_length as usize);

            gl_call! { gl::GetProgramInfoLog(self.shader_program.id.as_primitive(), error_length, ptr::null_mut(), error_message.as_mut_ptr()); }

            let message =
                CString::from_vec_with_nul(error_message.into_iter().map(|x| x as u8).collect())
                    .unwrap_or_else(|_| c"no useful message".into());

            Err(ValidateError::Message(message))
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ShaderSetUniformError<'a> {
    NameNul(NulError),
    NotFound {
        c_name: CString,
        shader_source: &'a str,
    },
}
error_boilerplate!(<'a> ShaderSetUniformError<'a>);

fn match_result(res: Result<(), SetUniformError<'_>>) -> Result<(), ShaderSetUniformError<'_>> {
    match res {
        Ok(()) | Err(SetUniformError::GetLocation(GetUniformLocationError::OptimizedOut)) => Ok(()),
        Err(SetUniformError::GetLocation(GetUniformLocationError::NotFound {
            c_name,
            shader_source,
        })) => Err(ShaderSetUniformError::NotFound {
            c_name,
            shader_source,
        }),
        Err(uniform::SetUniformError::GetLocation(GetUniformLocationError::NameNul(err))) => {
            Err(ShaderSetUniformError::NameNul(err))
        }
    }
}

impl<'a, M: Model, O, T: Texture, const OUT: usize> ActiveShaderProgram<'a, '_, '_, M, O, T, OUT> {
    pub fn set_uniform<U: Uniform + ?Sized, S: Into<String>>(
        &mut self,
        name: S,
        value: &U,
    ) -> Result<(), ShaderSetUniformError<'a>> {
        match_result(value.set_uniform(name.into(), self))
    }

    pub fn set_uniform_dyn(
        &mut self,
        values: Vec<(String, &dyn Uniform)>,
    ) -> Result<(), ShaderSetUniformError<'a>> {
        for (name, value) in values {
            match_result(value.set_uniform(name, self))?;
        }
        Ok(())
    }

    pub fn get_uniform_location(
        &self,
        name: String,
    ) -> Result<UniformLocation, GetUniformLocationError<'_>> {
        self.shader_program.get_uniform_location(name)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum BindTexturesError<'a> {
    BindTooHigh { maximum: usize },
    SetUniform(Vec<ShaderSetUniformError<'a>>),
}
error_boilerplate!(<'a> BindTexturesError<'a>);

impl<'a, M: Model, O, T: Texture, const OUT: usize> ActiveShaderProgram<'a, '_, '_, M, O, T, OUT> {
    pub(crate) fn bind_textures(&mut self) -> Result<(), BindTexturesError<'a>> {
        static MAX_TEX_UNITS: LazyLock<usize> = LazyLock::new(|| {
            let mut out = 0;
            gl_call! {
                gl::GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &raw mut out);
            }
            out as usize
        });

        if self.texture_list.len() <= *MAX_TEX_UNITS {
            let mut errors = Vec::new();

            for (index, (name, tex)) in self.texture_list.clone().into_iter().enumerate() {
                tex.bind_to(index as u32);
                let result = self.set_uniform(name, &(index as i32));
                match result {
                    Err(x) => errors.push(x),
                    Ok(()) => {}
                }
                //.map_err(|err| BindTexturesError::SetUniform(err))?;
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(BindTexturesError::SetUniform(errors))
            }
        } else {
            Err(BindTexturesError::BindTooHigh {
                maximum: *MAX_TEX_UNITS,
            })
        }
    }
}

impl<'a, M: Model, O, T: Texture, const OUT: usize> ActiveShaderProgram<'a, '_, '_, M, O, T, OUT> {
    pub fn cull_face(
        &mut self,
        cull_face: CullFace,
    ) -> Result<(), FaceCullingForcedByProgramError> {
        self.context.cull_face(cull_face)
    }

    // TODO: Drawing Skybox Token which is passed to SkyBox::draw
    pub fn drawing_skybox(&mut self, drawing_skybox: bool) {
        self.context.drawing_skybox(drawing_skybox);
    }
}

pub trait IsActiveShaderProgram<'a> {
    fn get_uniform_location(
        &self,
        name: String,
    ) -> Result<UniformLocation, GetUniformLocationError<'a>>;
}

impl<'a, 'b, 'c, M: Model, O, T: Texture, const OUT: usize> IsActiveShaderProgram<'a>
    for ActiveShaderProgram<'a, 'b, 'c, M, O, T, OUT>
{
    fn get_uniform_location(
        &self,
        name: String,
    ) -> Result<UniformLocation, GetUniformLocationError<'a>> {
        self.shader_program.get_uniform_location(name)
    }
}
