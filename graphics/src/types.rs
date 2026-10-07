use core::fmt::{self, Debug};
use core::num::NonZero;
use std::fs;
use std::io::Write as _;

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

        #[expect(
            clippy::allow_attributes,
            reason = "sometimes the macro released unused code"
        )]
        impl $name {
            #[allow(dead_code, reason = "May not be required on this invocation of the macro")]
            pub type Primitive = $type;

            #[allow(dead_code, reason = "May not be required on this invocation of the macro")]
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


        #[expect(
            clippy::allow_attributes,
            reason = "sometimes the macro released unused code"
        )]
        impl $name {
            #[allow(dead_code, reason = "May not be required on this invocation of the macro")]
            pub type Primitive = $type;

            pub(crate) const  fn as_primitive(&self) -> Self::Primitive {
                self.0.get()
            }
        }

        //impl !Send for $name {}
    };
}

nz_opaque!(TexDim: GLsizei, Clone, Copy);

impl TexDim {
    pub const fn from_non_zero(value: NonZero<Self::Primitive>) -> Self {
        Self(value)
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "unlikely that screen size is 24 bit long"
    )]
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
    pub const fn as_usize(self) -> usize {
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
pub struct GLError(pub(crate) GLenum);

impl Debug for GLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = if self.0 == gl::INVALID_OPERATION {
            "gl::INVALID_OPERATION"
        } else {
            let mut file = fs::File::options()
                .append(true)
                .create(true)
                .open("error_codes.log")
                .map_err(|_err| fmt::Error)?;
            writeln!(file, "{}", self.0).map_err(|_err| fmt::Error)?;
            &format!("unknown error: {}", self.0)
        };
        f.write_str(string)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VertexAttrType {
    #[doc(hidden)]
    GLfloat(u8),
    #[doc(hidden)]
    GLint(u8),
}

impl VertexAttrType {
    pub(crate) const fn get_enum(self) -> GLenum {
        match self {
            Self::GLfloat(_) => gl::FLOAT,
            Self::GLint(_) => gl::INT,
        }
    }
}

pub const trait VertexAttr: Copy {
    fn attr(val: u8) -> VertexAttrType;
}

const impl VertexAttr for GLfloat {
    fn attr(val: u8) -> VertexAttrType {
        VertexAttrType::GLfloat(val)
    }
}

const impl VertexAttr for GLint {
    fn attr(val: u8) -> VertexAttrType {
        VertexAttrType::GLint(val)
    }
}
