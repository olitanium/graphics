use std::path::Path;
use std::{fs, io};

use utils::error_boilerplate;

use crate::gl_call;
use crate::types::{GLenum, ShaderId};

#[derive(Debug)]
pub struct Shader {
    id: ShaderId,
    pub(crate) source: String,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum NewShaderError {
    Io(io::Error),
    NewFromSlice(NewFromSliceError),
}
error_boilerplate!(NewShaderError);

impl Shader {
    pub fn new<P: AsRef<Path>>(shader_type: GLenum, source: P) -> Result<Self, NewShaderError> {
        let source = source.as_ref();

        let shader_source = fs::read_to_string(source).map_err(|err| NewShaderError::Io(err))?;

        Self::new_from_slice(shader_type, shader_source.as_bytes())
            .map_err(|err| NewShaderError::NewFromSlice(err))
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum NewFromSliceError {
    SourceTooLong,
    CompileError,
}
error_boilerplate!(NewFromSliceError);

impl Shader {
    pub(crate) fn new_from_slice(
        shader_type: GLenum,
        cstr: &[u8],
    ) -> Result<Self, NewFromSliceError> {
        let shader_id = ShaderId::new(gl_call! { gl::CreateShader(shader_type) });

        let len = cstr
            .len()
            .try_into()
            .map_err(|_| NewFromSliceError::SourceTooLong)?;

        let ptr_len = &raw const len;
        let ptr_cstr = cstr.as_ptr().cast();

        gl_call! {
            gl::ShaderSource(shader_id.as_primitive(), 1, &raw const ptr_cstr, ptr_len);
        }
        gl_call! {
            gl::CompileShader(shader_id.as_primitive());
        }

        let mut is_success = 0;
        gl_call! {
            gl::GetShaderiv(
                shader_id.as_primitive(),
                gl::COMPILE_STATUS,
                &raw mut is_success,
            );
        }

        if is_success != gl::TRUE.into() {
            // TODO: find error reasons
            Err(NewFromSliceError::CompileError)
        } else {
            let source = String::from_utf8_lossy_owned(cstr.to_vec());
            Ok(Self {
                id: shader_id,
                source,
            })
        }
    }

    pub fn id(&self) -> &ShaderId {
        &self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        gl_call! { gl::DeleteShader(self.id.as_primitive()) }
    }
}
