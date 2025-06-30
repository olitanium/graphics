use std::fs;
use std::path::Path;

use super::error::Error;
use crate::types::{GLenum, ShaderId, ToPrimitive};
use crate::{gl_call, error::Result};

#[derive(Debug)]
pub struct Shader {
    id: ShaderId,
    pub(crate) source: String,
}

impl Shader {
    pub fn new<P: AsRef<Path>>(shader_type: GLenum, source: P) -> Result<Self> {
        let source = source.as_ref();

        let shader_source = fs::read_to_string(source).map_err(|_| Error::NoSourceFile {
            path: source.into(),
        })?;

        Self::new_from_slice(shader_type, shader_source.as_bytes())
    }

    pub(crate) fn new_from_slice(shader_type: GLenum, cstr: &[u8]) -> Result<Self> {
        let shader_id = ShaderId::new(gl_call! { gl::CreateShader(shader_type) });

        let len = [cstr.len().try_into().map_err(|_| Error::SourceTooLong {
            source: String::from_utf8_lossy(cstr).into_owned(),
            len: cstr.len(),
        })?];
        let arr_cstr = [cstr.as_ptr().cast()];

        gl_call! {
            gl::ShaderSource(shader_id.to_primitive(), 1, arr_cstr.as_ptr(), len.as_ptr());
        }
        gl_call! {
            gl::CompileShader(shader_id.to_primitive());
        }

        let mut is_success = 0;
        gl_call! {
            gl::GetShaderiv(
                shader_id.to_primitive(),
                gl::COMPILE_STATUS,
                &raw mut is_success,
            );
        }

        if is_success != gl::TRUE.into() {
            Err(Error::CompileError {
                source: String::from_utf8_lossy(cstr).into_owned(),
            }
            .into())
        } else {
            Ok(Self {
                id: shader_id,
                source: String::from_utf8_lossy_owned(cstr.to_vec()),
            })
        }
    }

    pub fn id(&self) -> &ShaderId {
        &self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        gl_call! { gl::DeleteShader(self.id.to_primitive()) }
    }
}
