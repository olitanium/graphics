use std::fmt::Debug;
use std::io::Write;
use std::num::{NonZero, NonZeroUsize};

pub(crate) use gl::types::*;

macro_rules! opaque {
    ($name:ident : $type:ident $(, $derives: ident)* $(,)?) => {
        #[derive(Debug, Hash, PartialEq, Eq $(, $derives)*)]
        pub struct $name($type);
        // TODO: make inner a Self::Primitive
        // Currently this makes rust-analyzer unhappy

        impl From<$type> for $name {
            fn from(value: Self::Primitive) -> Self {
                Self::new(value)
            }
        }

        impl $name {
            #[allow(dead_code)]
            pub type Primitive = $type;

            #[allow(dead_code)]
            pub(crate) const fn as_primitive(&self) -> Self::Primitive {
                self.0
            }
        }

        impl $name {
            pub const fn new(value: Self::Primitive) -> Self {
                Self(value)
            }
        }

        //impl !Send for $name {}
    };
}

macro_rules! nz_opaque {
    ($name:ident : $type:ident $(, $derives: ident)* $(,)?) => {
        #[derive(Debug, Hash, PartialEq, Eq $(, $derives)*)]
        pub struct $name(NonZero<$type>);

        impl $name {
            pub(crate) const fn new(value: Self::Primitive) -> Self {
                $name(NonZero::new(value).unwrap())
            }
        }

        /*impl $name {
            pub(crate) fn try_new(value: Self::Primitive) -> Option<Self> {
                NonZero::new(value).map($name)
            }
        }*/

        impl $name {
            #[expect(dead_code)]
            pub(crate) type Primitive = $type;

            pub(crate) const  fn as_primitive(&self) -> Self::Primitive {
                self.0.get()
            }
        }

        //impl !Send for $name {}
    };
}

nz_opaque!(TexDim: GLsizei, Clone, Copy);

impl TexDim {
    pub const fn from_non_zero(value: NonZeroUsize) -> Self {
        Self(NonZero::new(value.get() as _).unwrap())
    }

    pub const fn aspect_ratio(x: Self, y: Self) -> f32 {
        x.as_primitive() as f32 / y.as_primitive() as f32
    }
}
opaque!(TexId: GLuint);
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
nz_opaque!(Query: GLuint);

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

#[expect(non_camel_case_types, reason = "gl crate convention")]
#[derive(Debug, Clone, Copy)]
pub enum VertexAttrType {
    #[doc(hidden)]
    GLfloat(u8),
    #[doc(hidden)]
    GLint(u8),
}

impl VertexAttrType {
    pub(crate) fn get_enum(self) -> gl::types::GLenum {
        match self {
            Self::GLfloat(_) => gl::FLOAT,
            Self::GLint(_) => gl::INT,
        }
    }
}

pub const trait VertexAttr: Copy {
    fn attr(val: u8) -> VertexAttrType;
}

const impl VertexAttr for gl::types::GLfloat {
    fn attr(val: u8) -> VertexAttrType {
        VertexAttrType::GLfloat(val)
    }
}

const impl VertexAttr for gl::types::GLint {
    fn attr(val: u8) -> VertexAttrType {
        VertexAttrType::GLint(val)
    }
}
