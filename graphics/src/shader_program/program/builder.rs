use std::ffi::CString;
use std::marker::PhantomData;
use std::mem;
use std::path::Path;

use utils::{builder, new};

use super::{CullFace, ShaderProgram};
use crate::error::ApiResult;
use crate::gl_call;
use crate::shader_program::Error;
use crate::shader_program::shader::Shader;
use crate::texture::Texture;
use crate::types::{self, ShaderProgramId};
use crate::vertex::Vertex;

#[derive(Debug)]
pub struct VertexShader(Shader);
#[derive(Debug, Default)]
pub struct MissingVertexShader;

#[derive(Debug)]
pub struct FragmentShader(Shader);
#[derive(Debug, Default)]
pub struct MissingFragmentShader;

#[derive(Debug)]
pub struct Builder<Vx: Vertex, M: ?Sized, T: Texture, const OUT: usize, V, F> {
    vertex_shader: V,
    fragment_shader: F,
    geometry_shader: Option<Shader>,
    force_cull_face: Option<CullFace>,
    _phantom_model: PhantomData<fn(M, Vx, T)>,
}

impl<Vx: Vertex, M: ?Sized, T: Texture, const OUT: usize> Default
    for Builder<Vx, M, T, OUT, MissingVertexShader, MissingFragmentShader>
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

impl<Vx: Vertex, M: ?Sized, T: Texture, const OUT: usize>
    Builder<Vx, M, T, OUT, MissingVertexShader, MissingFragmentShader>
{
    new!();
}

impl<Vx: Vertex, M: ?Sized, T: Texture, const OUT: usize, V, F> Builder<Vx, M, T, OUT, V, F> {
    builder!(force_cull_face: Option<CullFace>);

    pub fn vertex_shader<NEVER, P: AsRef<Path>>(
        self,
        source: P,
    ) -> ApiResult<Builder<Vx, M, T, OUT, VertexShader, F>, NEVER> {
        Shader::new(gl::VERTEX_SHADER, source).map(|shader| Builder {
            vertex_shader: VertexShader(shader),
            ..self
        })
    }

    pub fn vertex_shader_raw<NEVER>(
        self,
        source: &[u8],
    ) -> ApiResult<Builder<Vx, M, T, OUT, VertexShader, F>, NEVER> {
        Shader::new_from_slice(gl::VERTEX_SHADER, source).map(|shader| Builder {
            vertex_shader: VertexShader(shader),
            ..self
        })
    }

    pub fn fragment_shader<NEVER, P: AsRef<Path>>(
        self,
        source: P,
    ) -> ApiResult<Builder<Vx, M, T, OUT, V, FragmentShader>, NEVER> {
        Shader::new(gl::FRAGMENT_SHADER, source).map(|shader| Builder {
            fragment_shader: FragmentShader(shader),
            ..self
        })
    }

    pub fn fragment_shader_raw<NEVER>(
        self,
        source: &[u8],
    ) -> ApiResult<Builder<Vx, M, T, OUT, V, FragmentShader>, NEVER> {
        Shader::new_from_slice(gl::FRAGMENT_SHADER, source).map(|shader| Builder {
            fragment_shader: FragmentShader(shader),
            ..self
        })
    }

    pub fn geometry_shader<NEVER, P: AsRef<Path>>(self, source: P) -> ApiResult<Self, NEVER> {
        Shader::new(gl::GEOMETRY_SHADER, source).map(|shader| Builder {
            geometry_shader: Some(shader),
            ..self
        })
    }

    pub fn geometry_shader_raw<NEVER>(self, source: &[u8]) -> ApiResult<Self, NEVER> {
        Shader::new_from_slice(gl::GEOMETRY_SHADER, source).map(|shader| Builder {
            geometry_shader: Some(shader),
            ..self
        })
    }
}
impl<Vx: Vertex, M: ?Sized, T: Texture, const OUT: usize>
    Builder<Vx, M, T, OUT, VertexShader, FragmentShader>
{
    pub fn build<NEVER>(mut self) -> ApiResult<ShaderProgram<Vx, M, OUT, T>, NEVER> {
        let program_id = gl_call! { ShaderProgramId::new(gl::CreateProgram()) };
        let mut total_string = String::default();
        gl_call! {
            gl::AttachShader(program_id.to_primitive(), self.vertex_shader.0.id().to_primitive());
        }
        total_string.push_str(&self.vertex_shader.0.source);
        gl_call! {
            gl::AttachShader(program_id.to_primitive(), self.fragment_shader.0.id().to_primitive());
        }
        total_string.push_str(&self.fragment_shader.0.source);
        if let Some(geometry) = &self.geometry_shader {
            gl_call! {
                gl::AttachShader(program_id.to_primitive(), geometry.id().to_primitive());
            }
            total_string.push_str(&geometry.source);
        }
        gl_call! {
            gl::LinkProgram(program_id.to_primitive());
        }

        let mut number = 0;
        gl_call! {
            gl::GetProgramiv(program_id.to_primitive(), gl::LINK_STATUS, &raw mut number);
        }

        if number != gl::TRUE as i32 {
            const BUFF_SIZE: types::GLsizei = 1000;
            let mut v = vec![0; BUFF_SIZE as usize + 1];
            let ptr = v.as_mut_ptr().cast();
            let mut len = 0;
            gl_call! { gl::GetProgramInfoLog(program_id.to_primitive(), BUFF_SIZE, &raw mut len, ptr); }

            v.truncate(len as usize);
            let info_log = String::from_utf8(v).expect("ascii is a subset of UTF-8");

            Err(Error::ProgramLinkError {
                info_log,
                vertex_shader: mem::take(&mut self.vertex_shader.0.source),
                fragment_shader: mem::take(&mut self.fragment_shader.0.source),
                geometry_shader: self
                    .geometry_shader
                    .map(|mut shader| mem::take(&mut shader.source)),
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
