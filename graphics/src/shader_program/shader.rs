use alloc::string::FromUtf8Error;
use core::num::TryFromIntError;
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
    pub fn new<P>(shader_type: GLenum, source: P) -> Result<Self, NewShaderError>
    where
        P: AsRef<Path>,
    {
        let source = source.as_ref();

        let shader_source = fs::read_to_string(source).map_err(NewShaderError::Io)?;

        Self::new_from_slice(shader_type, shader_source.as_bytes())
            .map_err(NewShaderError::NewFromSlice)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum NewFromSliceError {
    SourceTooLong(TryFromIntError),
    CompileUtf8Lossy(FromUtf8Error),
    Compile(String),
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
            .map_err(NewFromSliceError::SourceTooLong)?;

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

        if is_success == gl::TRUE.into() {
            let source = String::from_utf8_lossy_owned(cstr.to_vec());
            Ok(Self {
                id: shader_id,
                source,
            })
        } else {
            let mut log_size = 0;
            gl_call! {gl::GetShaderiv(shader_id.as_primitive(), gl::INFO_LOG_LENGTH, &raw mut log_size); }

            #[expect(
                clippy::cast_sign_loss,
                reason = "`gl::GetShaderiv` will never return a negative"
            )]
            let mut log = vec![0_u8; log_size as usize];
            let mut actual_written = 0;
            gl_call! {gl::GetShaderInfoLog(shader_id.as_primitive(), log_size, &raw mut actual_written, log.as_mut_ptr().cast())};

            #[expect(
                clippy::cast_sign_loss,
                reason = "`gl::GetShaderiv` will never return a negative"
            )]
            log.truncate(actual_written as usize);

            let string_error =
                String::from_utf8(log).map_err(NewFromSliceError::CompileUtf8Lossy)?;
            // TODO: find error reasons
            Err(NewFromSliceError::Compile(string_error))
        }
    }

    pub const fn id(&self) -> &ShaderId {
        &self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        gl_call! { gl::DeleteShader(self.id.as_primitive()) }
    }
}
