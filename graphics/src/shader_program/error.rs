use std::ffi::CString;
use std::path::PathBuf;

use utils::error_boilerplate;

use super::CullFace;

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
    NulInUnformName {
        name: String,
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
}

error_boilerplate!(Error);

impl From<Error> for crate::error::Error {
    fn from(value: Error) -> Self {
        Self::Shader(value)
    }
}
