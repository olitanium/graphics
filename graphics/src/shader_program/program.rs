use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CString, NulError};
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::texture::Texture;
use crate::types::{ShaderProgramId, UniformLocation};
use crate::vertex::Vertex;
use crate::{Draw, Model, gl_call};

mod active_shader;
mod utils;
pub use active_shader::{
    ActiveShaderProgram,
    BindTexturesError,
    IsActiveShaderProgram,
    SetUniformError,
    ValidateError,
};

mod context;
pub use context::{FaceCullingForcedByProgramError, ShaderProgramContext};

mod builder;
pub use builder::Builder;
use builder::{MissingFragmentShader, MissingVertexShader};
mod uniform;
pub use uniform::{NameNotFoundError, Uniform};

mod cull_face;
use ::utils::error_boilerplate;
pub use cull_face::CullFace;

pub struct ShaderProgram<M: Model, O, const OUT: usize, T: Texture> {
    pub(crate) id: ShaderProgramId,
    concat_all_text: String,
    uniform_locations: RefCell<HashMap<CString, i32>>,
    force_cull_face: Option<CullFace>,
    _phantom_model: PhantomData<fn(M, O, T)>,
}

impl<M: Model, O, const OUT: usize, T: Texture> Debug for ShaderProgram<M, O, OUT, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        gl_call! {
            gl::UseProgram(self.id.as_primitive());
        }

        marker.force_cull_face(self.force_cull_face);

        ActiveShaderProgram::new(self, marker)
    }
}

#[derive(Debug)]
pub enum GetUniformLocationError<'a> {
    NameNul(NulError),
    NotFound {
        c_name: CString,
        shader_source: &'a str,
    },
}
error_boilerplate!(<'a> GetUniformLocationError<'a>);

impl<M: Model, O, const OUT: usize, T: Texture> ShaderProgram<M, O, OUT, T> {
    pub(crate) fn get_uniform_location(
        &self,
        name: String,
    ) -> Result<UniformLocation, GetUniformLocationError<'_>> {
        let c_name = CString::new(name).map_err(|err| GetUniformLocationError::NameNul(err))?;

        let out = *self
            .uniform_locations
            .borrow_mut()
            .entry(c_name.clone())
            .or_insert_with_key(|c_name| {
                gl_call! { gl::GetUniformLocation(self.id.as_primitive(), c_name.as_ptr()) }
            });

        if out != -1 {
            Ok(UniformLocation::new(out))
        } else {
            Err(GetUniformLocationError::NotFound {
                c_name,
                shader_source: &self.concat_all_text,
            })
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
