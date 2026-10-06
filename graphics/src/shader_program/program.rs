use alloc::ffi::{CString, NulError};
use alloc::sync::Arc;
use core::cell::RefCell;
use core::fmt::{self, Debug};
use core::hash::Hasher as _;
use core::marker::PhantomData;
use std::collections::HashMap;

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
mod uniform_name;
pub use uniform_name::UniformName;

mod cull_face;
use ::utils::error_boilerplate;
pub use cull_face::CullFace;

pub struct ShaderProgram<M: Model, O, const OUT: usize, T: Texture> {
    pub(crate) id: ShaderProgramId,
    concat_all_text: Arc<str>,
    uniform_locations: RefCell<HashMap<u64, i32>>,
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

// SAFETY:
// This is safe because only one `ActiveShaderProgram` can exist.
// which is the only way of accessing the internal `RefCell`.
unsafe impl<M: Model, O, const OUT: usize, T: Texture> Sync for ShaderProgram<M, O, OUT, T> {}

impl<M: Model, O, const OUT: usize, T: Texture> ShaderProgram<M, O, OUT, T> {
    pub fn builder() -> Builder<M, O, T, OUT, MissingVertexShader, MissingFragmentShader> {
        Builder::new()
    }

    pub const fn id(&self) -> &ShaderProgramId {
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
pub enum GetUniformLocationError {
    NameNul(NulError),
    NotFound {
        c_name: CString,
        shader_source: Arc<str>,
    },
    OptimizedOut,
}
error_boilerplate!(GetUniformLocationError);

impl<M: Model, O, const OUT: usize, T: Texture> ShaderProgram<M, O, OUT, T> {
    pub(crate) fn get_uniform_location(
        &self,
        name: &dyn UniformName,
    ) -> Result<UniformLocation, GetUniformLocationError> {
        let hash = name.hash().finish();

        let out = *self
            .uniform_locations
            .borrow_mut()
            .entry(hash)
            .or_insert_with(|| {
                let c_name = name.as_cstr();
                let mut location =
                    gl_call! { gl::GetUniformLocation(self.id.as_primitive(), c_name.as_ptr()) };
                if location == -1 {
                    let name = c_name.to_string_lossy();
                    let split_on_index = name.split('[').next();

                    let name_before_index = split_on_index.unwrap_or_else(|| name.as_ref());
                    let is_it_in_the_source = self.concat_all_text.contains(name_before_index);
                    location = if is_it_in_the_source { -2 } else { -1 };
                }
                location
            });

        // if shader_source contains c_name then
        match out {
            -2 => Err(GetUniformLocationError::OptimizedOut),
            -1 => Err(GetUniformLocationError::NotFound {
                c_name: name.as_cstr().to_owned(),
                shader_source: Arc::clone(&self.concat_all_text),
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
