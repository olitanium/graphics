use std::marker::PhantomData;
use std::path::Path;

use utils::{builder, new};

use super::{CullFace, ShaderProgram};
use crate::error::Result;
use crate::gl_call;
use crate::shader_program::shader::Shader;
use crate::texture::Texture;
use crate::types::{self, ShaderProgramId, ToPrimitive};

#[derive(Debug)]
pub struct VertexShader(Shader);
#[derive(Debug, Default)]
pub struct MissingVertexShader;

#[derive(Debug)]
pub struct FragmentShader(Shader);
#[derive(Debug, Default)]
pub struct MissingFragmentShader;

#[derive(Debug)]
pub struct Builder<M, T: Texture, const OUT: usize, V, F> {
    vertex_shader: V,
    fragment_shader: F,
    geometry_shader: Option<Shader>,
    force_cull_face: Option<CullFace>,
    _phantom_model: PhantomData<fn(M)>,
    _phantom_tex: PhantomData<fn(T)>,
}

impl<M, T: Texture, const OUT: usize> Default
    for Builder<M, T, OUT, MissingVertexShader, MissingFragmentShader>
{
    fn default() -> Self {
        Self {
            vertex_shader: MissingVertexShader,
            fragment_shader: MissingFragmentShader,
            geometry_shader: None,
            force_cull_face: None,
            _phantom_model: PhantomData,
            _phantom_tex: PhantomData,
        }
    }
}

impl<M, T: Texture, const OUT: usize>
    Builder<M, T, OUT, MissingVertexShader, MissingFragmentShader>
{
    new!();
}

impl<M, T: Texture, const OUT: usize, V, F> Builder<M, T, OUT, V, F> {
    builder!(force_cull_face: Option<CullFace>);

    #[inline]
    pub fn vertex_shader<P: AsRef<Path>>(
        self,
        source: P,
    ) -> Result<Builder<M, T, OUT, VertexShader, F>> {
        Shader::new(gl::VERTEX_SHADER, source).map(|shader| Builder {
            vertex_shader: VertexShader(shader),
            ..self
        })
    }

    #[inline]
    pub fn vertex_shader_raw(self, source: &[u8]) -> Result<Builder<M, T, OUT, VertexShader, F>> {
        Shader::new_from_slice(gl::VERTEX_SHADER, source).map(|shader| Builder {
            vertex_shader: VertexShader(shader),
            ..self
        })
    }

    #[inline]
    pub fn fragment_shader<P: AsRef<Path>>(
        self,
        source: P,
    ) -> Result<Builder<M, T, OUT, V, FragmentShader>> {
        Shader::new(gl::FRAGMENT_SHADER, source).map(|shader| Builder {
            fragment_shader: FragmentShader(shader),
            ..self
        })
    }

    #[inline]
    pub fn fragment_shader_raw(
        self,
        source: &[u8],
    ) -> Result<Builder<M, T, OUT, V, FragmentShader>> {
        Shader::new_from_slice(gl::FRAGMENT_SHADER, source).map(|shader| Builder {
            fragment_shader: FragmentShader(shader),
            ..self
        })
    }

    pub fn geometry_shader<P: AsRef<Path>>(self, source: P) -> Result<Self> {
        Shader::new(gl::GEOMETRY_SHADER, source).map(|shader| Builder {
            geometry_shader: Some(shader),
            ..self
        })
    }

    pub fn geometry_shader_raw(self, source: &[u8]) -> Result<Self> {
        Shader::new_from_slice(gl::GEOMETRY_SHADER, source).map(|shader| Builder {
            geometry_shader: Some(shader),
            ..self
        })
    }
}
impl<M, T: Texture, const OUT: usize> Builder<M, T, OUT, VertexShader, FragmentShader> {
    #[inline]
    pub fn build(self) -> ShaderProgram<M, OUT, T> {
        let program_id = gl_call! { ShaderProgramId::new(gl::CreateProgram()) };

        gl_call! {
            gl::AttachShader(program_id.to_primitive(), self.vertex_shader.0.id().to_primitive());
        }
        gl_call! {
            gl::AttachShader(program_id.to_primitive(), self.fragment_shader.0.id().to_primitive());
        }
        if let Some(geometry) = &self.geometry_shader {
            gl_call! {
                gl::AttachShader(program_id.to_primitive(), geometry.id().to_primitive());
            }
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

            eprintln!("{info_log}");

            panic!(
                "failed to link shader:\nVertex Shader:\n{}\n\nFragment Shader:\n{}\n\nGeometry \
                 Shader:\n{}",
                self.vertex_shader.0.source,
                self.fragment_shader.0.source,
                self.geometry_shader
                    .map_or("None".into(), |shader| shader.source.clone())
            );
        }

        ShaderProgram {
            id: program_id,
            uniform_locations: Default::default(),
            force_cull_face: self.force_cull_face,
            _phantom_model: PhantomData,
            _phantom_tex: PhantomData,
        }
    }
}
