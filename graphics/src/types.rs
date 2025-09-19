use std::fmt::Debug;
use std::io::Write;
use std::num::NonZero;

pub(crate) use gl::types::*;

macro_rules! opaque {
    ($name:ident : $type:ident $(, $derives: ident)* $(,)?) => {
        #[derive(Debug, Hash, PartialEq, Eq $(, $derives)*)]
        pub struct $name(gl::types::$type);
        // TODO: make inner a Self::Primitive
        // Currently this makes rust-analyzer unhappy

        impl From<gl::types::$type> for $name {
            fn from(value: Self::Primitive) -> Self {
                Self::new(value)
            }
        }

        impl $name {
            #[allow(dead_code)]
            pub type Primitive = gl::types::$type;

            #[allow(dead_code)]
            pub(crate) fn to_primitive(&self) -> Self::Primitive {
                self.0
            }
        }

        impl $name {
            pub const fn new(value: Self::Primitive) -> Self {
                Self(value)
            }
        }
    };
}

macro_rules! nz_opaque {
    ($name:ident : $type:ident $(, $derives: ident)* $(,)?) => {
        #[derive(Debug, Hash, PartialEq, Eq $(, $derives)*)]
        pub struct $name(NonZero<gl::types::$type>);

        impl From<gl::types::$type> for $name {
            fn from(value: Self::Primitive) -> Self {
                Self::new(value)
            }
        }

        impl $name {
            pub(crate) fn new(value: Self::Primitive) -> Self {
                $name(NonZero::new(value).unwrap())
            }
        }

        impl $name {
            pub fn try_new(value: Self::Primitive) -> Option<Self> {
                NonZero::new(value).map($name)
            }
        }

        impl $name {
            pub type Primitive = gl::types::$type;

            pub(crate) fn to_primitive(&self) -> Self::Primitive {
                self.0.get()
            }
        }
    };
}

nz_opaque!(TexDim: GLsizei, Clone, Copy);
nz_opaque!(TexId: GLuint);
opaque!(FrameBufferId: GLuint);
nz_opaque!(VertexArrayId: GLuint);
nz_opaque!(ElementArrayId: GLuint);
opaque!(ElementArrayElem: GLuint, Clone, Copy);

impl ElementArrayElem {
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}
opaque!(ElementArrayLen: GLsizei, Clone, Copy);
opaque!(VertexBufferLen: GLsizei, Clone, Copy);
nz_opaque!(VertexBufferId: GLuint);
opaque!(UniformLocation: GLint);
nz_opaque!(ShaderId: GLuint);
nz_opaque!(ShaderProgramId: GLuint);

#[derive(Clone, Copy)]
pub struct GLError(pub(crate) gl::types::GLenum);

impl Debug for GLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error;
        f.write_str(match self.0 {
            gl::INVALID_OPERATION => stringify!(gl::INVALID_OPERATION),
            x => {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open("error_codes.log")
                    .unwrap();
                let _ = writeln!(file, "{x}");
                error = format!("unknown error: {}", self.0);
                &error
            }
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum VertexAttrType {
    f32,
    i32,
}

impl VertexAttrType {
    pub(crate) fn get_enum(self) -> gl::types::GLenum {
        match self {
            Self::f32 => gl::FLOAT,
            Self::i32 => gl::INT,
        }
    }
}
