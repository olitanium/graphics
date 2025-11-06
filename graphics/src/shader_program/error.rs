use std::ffi::CString;
use std::path::PathBuf;

use utils::error_boilerplate;

use super::CullFace;
use crate::ApiError;

#[derive(Debug, Clone)]
pub enum Error {
    NoSourceFile {
        path: PathBuf,
    },
    NulError {
        source: String,
    },
    SourceTooLong {
        source: String,
        len: usize,
    },
    CompileError {
        source: String,
    },
    TooLongVecErrorInt {
        length_given: usize,
        vector: Vec<i32>,
    },
    TooLongVecErrorFloat {
        length_given: usize,
        vector: Vec<f32>,
    },
    Validate {
        message: CString,
    },
    TriedToCullFaceWhenFaceCullingForcedByProgram {
        forced: CullFace,
        attempted: CullFace,
    },
    ProgramLinkError {
        info_log: String,
        vertex_shader: String,
        fragment_shader: String,
        geometry_shader: Option<String>,
    },
    UniformNotFound {
        shader_source: String,
        name: CString,
    },
    UniformNameNulError {
        name: String,
    },
}

error_boilerplate!(Error);

impl<E> From<Error> for ApiError<E> {
    fn from(value: Error) -> Self {
        Self::Shader(value)
    }
}
