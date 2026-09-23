use core::cell::RefCell;
use core::fmt::{self, Debug};
use core::marker::PhantomData;
use std::collections::HashMap;
use std::ffi::{CString, NulError};

use crate::texture::Texture;
use crate::types::{ShaderProgramId, UniformLocation};
use crate::{Model, gl_call};

mod active_shader;
mod utils;
pub use active_shader::{
    ActiveShaderProgram,
    BindTexturesError,
    IsActiveShaderProgram,
    ShaderSetUniformError,
    ValidateError,
};

mod context;
pub use context::{FaceCullingForcedByProgramError, ShaderProgramContext};

mod builder;
pub use builder::Builder;
use builder::{MissingFragmentShader, MissingVertexShader};
mod uniform;
pub use uniform::{SetUniformError, Uniform};

mod cull_face;
use ::utils::error_boilerplate;
pub use cull_face::CullFace;

pub struct ShaderProgram<M: Model, O, const OUT: usize, T: Texture> {
    pub(crate) id: ShaderProgramId,
    concat_all_text: String,
    uniform_locations: RefCell<HashMap<CString, i32>>,
    force_cull_face: Option<CullFace>,
    _phantom_model: PhantomData<fn(M, T) -> O>,
}

impl<M: Model, O, const OUT: usize, T: Texture> Debug for ShaderProgram<M, O, OUT, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(ShaderProgram))
            .field("concat_all_text", &self.concat_all_text)
            .finish_non_exhaustive()
    }
}

/// # SAFETY
/// This is safe because only one ActiveShaderProgram can exist,
/// which is the only way of accessing the internal RefCell
unsafe impl<M: Model, O, const OUT: usize, T: Texture> Sync for ShaderProgram<M, O, OUT, T> {}

impl<M: Model, O, const OUT: usize, T: Texture> ShaderProgram<M, O, OUT, T> {
    pub fn builder() -> Builder<M, O, T, OUT, MissingVertexShader, MissingFragmentShader> {
        Builder::new()
    }

    pub fn id(&self) -> &ShaderProgramId {
        &self.id
    }

    pub fn use_program<'a, 'b, 'c>(
        &'a self,
        marker: &'b mut ShaderProgramContext,
    ) -> ActiveShaderProgram<'a, 'b, 'c, M, O, T, OUT> {
        marker.use_program(self);
        ActiveShaderProgram::new(self, marker)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum GetUniformLocationError<'a> {
    NameNul(NulError),
    NotFound {
        c_name: CString,
        shader_source: &'a str,
    },
    OptimizedOut,
}
error_boilerplate!(<'a> GetUniformLocationError<'a>);

impl<M: Model, O, const OUT: usize, T: Texture> ShaderProgram<M, O, OUT, T> {
    pub(crate) fn get_uniform_location(
        &self,
        name: String,
    ) -> Result<UniformLocation, GetUniformLocationError<'_>> {
        let c_name = CString::new(name).map_err(GetUniformLocationError::NameNul)?;

        let out = *self
            .uniform_locations
            .borrow_mut()
            .entry(c_name.clone())
            .or_insert_with_key(|c_name| {
                let mut location =
                    gl_call! { gl::GetUniformLocation(self.id.as_primitive(), c_name.as_ptr()) };
                if location == -1 {
                    let is_it_in_the_source = self.concat_all_text.contains(&{
                        let name = c_name.to_string_lossy();
                        let opt_name_before_list_index = name.split('[').next();

                        opt_name_before_list_index
                            .unwrap_or(name.as_ref())
                            .to_owned()
                    });
                    location = if is_it_in_the_source { -2 } else { -1 };
                }
                location
            });

        // if shader_source contains c_name then
        match out {
            -2 => Err(GetUniformLocationError::OptimizedOut),
            -1 => Err(GetUniformLocationError::NotFound {
                c_name,
                shader_source: &self.concat_all_text,
            }),
            other => Ok(UniformLocation::new(other)),
        }
    }
}

impl<M: Model, O, T: Texture, const OUT: usize> Drop for ShaderProgram<M, O, OUT, T> {
    fn drop(&mut self) {
        gl_call! {
            gl::DeleteProgram(self.id.as_primitive());
        }
    }
}
