use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::marker::PhantomData;

pub use super::Error;
use crate::gl_call;
use crate::texture::Texture;
use crate::types::{ShaderProgramId, UniformLocation};

mod active_shader;
mod utils;
pub use active_shader::ActiveShaderProgram;

mod context;
pub use context::ShaderProgramContext;

mod builder;
pub use builder::Builder;
use builder::{MissingFragmentShader, MissingVertexShader};
mod uniform;
pub use uniform::Uniform;

mod cull_face;
pub use cull_face::CullFace;

#[derive(Debug)]
pub struct ShaderProgram<M, const OUT: usize, T: Texture> {
    pub(crate) id: ShaderProgramId,
    uniform_locations: RefCell<HashMap<CString, i32>>,
    force_cull_face: Option<CullFace>,
    _phantom_model: PhantomData<fn(M)>,
    _phantom_tex: PhantomData<fn(T)>,
}

/// # SAFETY
/// This is safe because only one ActiveShaderProgram can exist,
/// which is the only way of accessing the internal RefCell
unsafe impl<M, const OUT: usize, T: Texture> Sync for ShaderProgram<M, OUT, T> {}

impl<M, const OUT: usize, T: Texture> ShaderProgram<M, OUT, T> {
    pub fn builder() -> Builder<M, T, OUT, MissingVertexShader, MissingFragmentShader> {
        Builder::new()
    }

    pub fn id(&self) -> &ShaderProgramId {
        &self.id
    }

    pub fn use_program<'a, 'b, 'c>(
        &'a self,
        marker: &'b mut ShaderProgramContext,
    ) -> ActiveShaderProgram<'a, 'b, 'c, M, T, OUT> {
        gl_call! {
            gl::UseProgram(self.id.to_primitive());
        }

        marker.force_cull_face(self.force_cull_face);

        ActiveShaderProgram::new(self, marker)
    }

    pub(crate) fn get_uniform_location(&self, name: String) -> Option<UniformLocation> {
        let c_name = CString::new(name).ok()?;

        let out = *self
            .uniform_locations
            .borrow_mut()
            .entry(c_name)
            .or_insert_with_key(|c_name| {
                gl_call! { gl::GetUniformLocation(self.id.to_primitive(), c_name.as_ptr()) }
            });
        
        if out != -1 {
            Some(UniformLocation::new(out))
        } else {
            None
        }
    }
}

impl<M, T: Texture, const OUT: usize> Drop for ShaderProgram<M, OUT, T> {
    fn drop(&mut self) {
        gl_call! {
            gl::DeleteProgram(self.id.to_primitive());
        }
    }
}
