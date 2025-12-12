use std::marker::PhantomData;
use std::mem;
use std::path::Path;

use gl::LinkProgram;
use utils::{builder, error_boilerplate, new};

use super::{CullFace, ShaderProgram};
use crate::shader_program::shader::{NewFromSliceError, NewShaderError, Shader};
use crate::texture::Texture;
use crate::types::{self, ShaderProgramId};
use crate::{Model, gl_call};

#[derive(Debug)]
pub struct VertexShader(Shader);
#[derive(Debug, Default)]
pub struct MissingVertexShader;

#[derive(Debug)]
pub struct FragmentShader(Shader);
#[derive(Debug, Default)]
pub struct MissingFragmentShader;

#[derive(Debug)]
pub struct Builder<M: Model, O, T: Texture, const OUT: usize, V, F> {
    vertex_shader: V,
    fragment_shader: F,
    geometry_shader: Option<Shader>,
    force_cull_face: Option<CullFace>,
    _phantom_model: PhantomData<fn(M, O, T)>,
}

impl<M: Model, O, T: Texture, const OUT: usize> Default
    for Builder<M, O, T, OUT, MissingVertexShader, MissingFragmentShader>
{
    fn default() -> Self {
        Self {
            vertex_shader: MissingVertexShader,
            fragment_shader: MissingFragmentShader,
            geometry_shader: None,
            force_cull_face: None,
            _phantom_model: PhantomData,
        }
    }
}

impl<M: Model, O, T: Texture, const OUT: usize>
    Builder<M, O, T, OUT, MissingVertexShader, MissingFragmentShader>
{
    new!();
}

impl<M: Model, O, T: Texture, const OUT: usize, V, F> Builder<M, O, T, OUT, V, F> {
    builder!(force_cull_face: Option<CullFace>);

    pub fn vertex_shader<P: AsRef<Path>>(
        self,
        source: P,
    ) -> Result<Builder<M, O, T, OUT, VertexShader, F>, NewShaderError> {
        let shader = Shader::new(gl::VERTEX_SHADER, source)?;
        Ok(Builder {
            vertex_shader: VertexShader(shader),
            ..self
        })
    }

    pub fn vertex_shader_raw(
        self,
        source: &[u8],
    ) -> Result<Builder<M, O, T, OUT, VertexShader, F>, NewFromSliceError> {
        let shader = Shader::new_from_slice(gl::VERTEX_SHADER, source)?;
        Ok(Builder {
            vertex_shader: VertexShader(shader),
            ..self
        })
    }

    pub fn fragment_shader<P: AsRef<Path>>(
        self,
        source: P,
    ) -> Result<Builder<M, O, T, OUT, V, FragmentShader>, NewShaderError> {
        let shader = Shader::new(gl::FRAGMENT_SHADER, source)?;
        Ok(Builder {
            fragment_shader: FragmentShader(shader),
            ..self
        })
    }

    pub fn fragment_shader_raw(
        self,
        source: &[u8],
    ) -> Result<Builder<M, O, T, OUT, V, FragmentShader>, NewFromSliceError> {
        let shader = Shader::new_from_slice(gl::FRAGMENT_SHADER, source)?;
        Ok(Builder {
            fragment_shader: FragmentShader(shader),
            ..self
        })
    }

    pub fn geometry_shader<P: AsRef<Path>>(self, source: P) -> Result<Self, NewShaderError> {
        let shader = Shader::new(gl::GEOMETRY_SHADER, source)?;
        Ok(Builder {
            geometry_shader: Some(shader),
            ..self
        })
    }

    pub fn geometry_shader_raw(self, source: &[u8]) -> Result<Self, NewFromSliceError> {
        let shader = Shader::new_from_slice(gl::GEOMETRY_SHADER, source)?;
        Ok(Builder {
            geometry_shader: Some(shader),
            ..self
        })
    }
}

#[derive(Debug)]
pub struct LinkError {
    pub info_log: String,
    pub total_string: String,
}
error_boilerplate!(LinkError);

impl<M: Model, O, T: Texture, const OUT: usize>
    Builder<M, O, T, OUT, VertexShader, FragmentShader>
{
    pub fn build(self) -> Result<ShaderProgram<M, O, OUT, T>, LinkError> {
        let program_id = gl_call! { ShaderProgramId::new(gl::CreateProgram()) };

        let mut total_string = String::default();
        gl_call! {
            gl::AttachShader(program_id.as_primitive(), self.vertex_shader.0.id().as_primitive());
        }
        total_string.push_str(&self.vertex_shader.0.source);
        gl_call! {
            gl::AttachShader(program_id.as_primitive(), self.fragment_shader.0.id().as_primitive());
        }
        total_string.push_str(&self.fragment_shader.0.source);
        if let Some(geometry) = &self.geometry_shader {
            gl_call! {
                gl::AttachShader(program_id.as_primitive(), geometry.id().as_primitive());
            }
            total_string.push_str(&geometry.source);
        }
        gl_call! {
            gl::LinkProgram(program_id.as_primitive());
        }

        let mut number = 0;
        gl_call! {
            gl::GetProgramiv(program_id.as_primitive(), gl::LINK_STATUS, &raw mut number);
        }

        if number != gl::TRUE as i32 {
            const BUFF_SIZE: types::GLsizei = 1000;
            let mut v = vec![0; BUFF_SIZE as usize + 1];
            let ptr = v.as_mut_ptr().cast();
            let mut len = 0;
            gl_call! { gl::GetProgramInfoLog(program_id.as_primitive(), BUFF_SIZE, &raw mut len, ptr); }

            v.truncate(len as usize);
            let info_log = String::from_utf8(v).expect("ascii is a subset of UTF-8");

            Err(LinkError {
                info_log,
                total_string,
            }
            .into())
        } else {
            Ok(ShaderProgram {
                id: program_id,
                concat_all_text: total_string,
                uniform_locations: Default::default(),
                force_cull_face: self.force_cull_face,
                _phantom_model: PhantomData,
            })
        }
    }
}
