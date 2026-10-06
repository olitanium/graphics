use alloc::ffi::{CString, NulError};
use alloc::sync::Arc;
use core::ffi::c_char;
use core::fmt::{self, Debug};
use core::ptr;
use std::collections::HashMap;
use std::sync::LazyLock;

use utils::error_boilerplate;

use super::uniform::Uniform;
use super::{CullFace, ShaderProgram, ShaderProgramContext};
use crate::shader_program::program::context::FaceCullingForcedByProgramError;
use crate::shader_program::program::uniform::SetUniformError;
use crate::shader_program::program::{GetUniformLocationError, UniformName};
use crate::texture::Texture;
use crate::types::{self, UniformLocation};
use crate::{Model, gl_call};

pub struct ActiveShaderProgram<'a, 'b, 'c, M: Model, O, T: Texture, const OUT: usize> {
    shader_program: &'a ShaderProgram<M, O, OUT, T>,
    context: &'b mut ShaderProgramContext,
    texture_list: HashMap<CString, &'c dyn Texture>,
}

impl<M: Model + Debug, O, T: Texture + Debug, const OUT: usize> Debug
    for ActiveShaderProgram<'_, '_, '_, M, O, T, OUT>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    pub const fn context(&self) -> &ShaderProgramContext {
        self.context
    }

    pub fn register_texture<L>(&mut self, texture_list: L)
    where
        L: IntoIterator<Item = (CString, &'c dyn Texture)>,
    {
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

impl<M: Model, O, T: Texture, const OUT: usize> ActiveShaderProgram<'_, '_, '_, M, O, T, OUT> {
    pub fn validate(&self) -> Result<(), ValidateError> {
        gl_call! { gl::ValidateProgram(self.shader_program.id.as_primitive()); }

        let mut error = 0;
        gl_call! { gl::GetProgramiv(self.shader_program.id.as_primitive(), gl::VALIDATE_STATUS, &raw mut error); }

        if error == types::GLint::from(gl::TRUE) {
            Ok(())
        } else {
            let mut error_length = 0;
            gl_call! { gl::GetProgramiv(self.shader_program.id.as_primitive(), gl::INFO_LOG_LENGTH, &raw mut error_length); }

            // let mut error_message: Vec<i8> = iter::repeat_n(0, error_length
            // as usize).collect(); // Vec::with_capacity(error_length as
            // usize);
            #[expect(clippy::cast_sign_loss, reason = "message will not be negative")]
            let mut error_message = vec![0; error_length as usize];

            gl_call! { gl::GetProgramInfoLog(self.shader_program.id.as_primitive(), error_length, ptr::null_mut(), error_message.as_mut_ptr()); }

            let message = CString::from_vec_with_nul(
                error_message
                    .into_iter()
                    .map(c_char::cast_unsigned)
                    .collect(),
            )
            .unwrap_or_else(|_| c"no useful message".into());

            Err(ValidateError::Message(message))
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ShaderSetUniformError {
    NameNul(NulError),
    NotFound {
        c_name: CString,
        // I don't like the Arc, because this is not thread safe and so `Arc` is unneccesary. `Arc`
        // is used because some Shaders are in `LazyLocks`.
        shader_source: Arc<str>,
    },
}
error_boilerplate!(ShaderSetUniformError);

fn match_result(res: Result<(), SetUniformError>) -> Result<(), ShaderSetUniformError> {
    match res {
        Ok(()) | Err(SetUniformError::GetLocation(GetUniformLocationError::OptimizedOut)) => Ok(()),
        Err(SetUniformError::GetLocation(GetUniformLocationError::NotFound {
            c_name,
            shader_source,
        })) => Err(ShaderSetUniformError::NotFound {
            c_name,
            shader_source,
        }),
        Err(SetUniformError::GetLocation(GetUniformLocationError::NameNul(err))) => {
            Err(ShaderSetUniformError::NameNul(err))
        }
    }
}

impl<M: Model, O, T: Texture, const OUT: usize> ActiveShaderProgram<'_, '_, '_, M, O, T, OUT> {
    pub fn set_uniform<N, U>(&mut self, name: &N, value: &U) -> Result<(), ShaderSetUniformError>
    where
        N: UniformName,
        U: Uniform + ?Sized,
    {
        match_result(value.set_uniform(name, self))
    }

    pub fn set_uniform_dyn(
        &mut self,
        values: Vec<(CString, &dyn Uniform)>,
    ) -> Result<(), ShaderSetUniformError> {
        for (name, value) in values {
            match_result(value.set_uniform(&name, self))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum BindTexturesError {
    BindTooHigh { maximum: usize },
    SetUniform(Vec<ShaderSetUniformError>),
}
error_boilerplate!(BindTexturesError);

impl<M: Model, O, T: Texture, const OUT: usize> ActiveShaderProgram<'_, '_, '_, M, O, T, OUT> {
    pub(crate) fn bind_textures(&mut self) -> Result<(), BindTexturesError> {
        static MAX_TEX_UNITS: LazyLock<usize> = LazyLock::new(|| {
            let mut out = 0;
            gl_call! {
                gl::GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &raw mut out);
            }
            #[expect(
                clippy::expect_used,
                reason = "OpenGL guaranteed that this will be positive, OpenGL could have used an \
                          unsigned integer but chose not to"
            )]
            usize::try_from(out).expect("will not be negative")
        });

        if self.texture_list.len() <= *MAX_TEX_UNITS {
            let mut errors = Vec::new();

            for (index, (name, tex)) in self.texture_list.clone().into_iter().enumerate() {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "index <= u32::MAX because MAX_TEX_UNITS < u32::MAX"
                )]
                let i32_index = index as i32;
                tex.bind_to(i32_index.cast_unsigned());
                let result = self.set_uniform(&name, &i32_index);
                if let Err(x) = result {
                    errors.push(x);
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

impl<M: Model, O, T: Texture, const OUT: usize> ActiveShaderProgram<'_, '_, '_, M, O, T, OUT> {
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

pub trait IsActiveShaderProgram {
    fn get_uniform_location(
        &self,
        name: &dyn UniformName,
    ) -> Result<UniformLocation, GetUniformLocationError>;
}

impl<M: Model, O, T: Texture, const OUT: usize> IsActiveShaderProgram
    for ActiveShaderProgram<'_, '_, '_, M, O, T, OUT>
{
    fn get_uniform_location(
        &self,
        name: &dyn UniformName,
    ) -> Result<UniformLocation, GetUniformLocationError> {
        self.shader_program.get_uniform_location(name)
    }
}
