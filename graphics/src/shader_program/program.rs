use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::marker::PhantomData;

pub use super::Error;
use crate::texture::Texture;
use crate::types::{ShaderProgramId, UniformLocation};
use crate::vertex::Vertex;
use crate::{ApiResult, gl_call};

mod active_shader;
mod utils;
pub use active_shader::{ActiveShaderProgram, IsActiveShaderProgram};

mod context;
pub use context::ShaderProgramContext;

mod builder;
pub use builder::Builder;
use builder::{MissingFragmentShader, MissingVertexShader};
mod uniform;
pub use uniform::{NameNotFoundError, Uniform};

mod cull_face;
pub use cull_face::CullFace;

mod draw_state;
pub use draw_state::DrawState;

#[derive(Debug)]
pub struct ShaderProgram<V: Vertex, M: ?Sized, const OUT: usize, T: Texture> {
    pub(crate) id: ShaderProgramId,
    concat_all_text: String,
    uniform_locations: RefCell<HashMap<CString, i32>>,
    force_cull_face: Option<CullFace>,
    _phantom_model: PhantomData<fn(M, V, T)>,
}

/// # SAFETY
/// This is safe because only one ActiveShaderProgram can exist,
/// which is the only way of accessing the internal RefCell
unsafe impl<V: Vertex, M, const OUT: usize, T: Texture> Sync for ShaderProgram<V, M, OUT, T> {}

impl<V: Vertex, M: ?Sized, const OUT: usize, T: Texture> ShaderProgram<V, M, OUT, T> {
    pub fn builder() -> Builder<V, M, T, OUT, MissingVertexShader, MissingFragmentShader> {
        Builder::new()
    }

    pub fn id(&self) -> &ShaderProgramId {
        &self.id
    }

    pub fn use_program<'a, 'b, 'c>(
        &'a self,
        marker: &'b mut ShaderProgramContext,
    ) -> ActiveShaderProgram<'a, 'b, 'c, V, M, T, OUT> {
        gl_call! {
            gl::UseProgram(self.id.to_primitive());
        }

        marker.force_cull_face(self.force_cull_face);

        ActiveShaderProgram::new(self, marker)
    }

    pub(crate) fn get_uniform_location<NEVER>(
        &self,
        name: String,
    ) -> ApiResult<UniformLocation, NEVER> {
        let c_name = CString::new(name).map_err(|err| Error::UniformNameNulError {
            name: String::try_from(err.into_vec()).expect("vec came from `String`"),
        })?;

        let out = *self
            .uniform_locations
            .borrow_mut()
            .entry(c_name.clone())
            .or_insert_with_key(|c_name| {
                gl_call! { gl::GetUniformLocation(self.id.to_primitive(), c_name.as_ptr()) }
            });

        if out != -1 {
            Ok(UniformLocation::new(out))
        } else {
            Err(Error::UniformNotFound {
                shader_source: self.concat_all_text.clone(),
                name: c_name,
            }
            .into())
        }
    }
}

impl<V: Vertex, M: ?Sized, T: Texture, const OUT: usize> Drop for ShaderProgram<V, M, OUT, T> {
    fn drop(&mut self) {
        gl_call! {
            gl::DeleteProgram(self.id.to_primitive());
        }
    }
}
