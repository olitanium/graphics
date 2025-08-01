use std::sync::Mutex;

use super::{CullFace, Error, ShaderProgram};
use crate::error::Result;
use crate::gl_call;
use crate::texture::Texture;

#[derive(Debug)]
pub struct ShaderProgramContext {
    current_cull_face: CullFace,
    current_drawing_skybox: bool,

    forced_cull_face: Option<CullFace>,
}
static IS_INIT: Mutex<bool> = Mutex::new(false);

impl ShaderProgramContext {
    pub fn new() -> Option<Self> {
        let mut is_init = IS_INIT.lock().ok()?;
        if *is_init {
            None
        } else {
            *is_init = true;
            Some(ShaderProgramContext {
                forced_cull_face: None,
                current_cull_face: CullFace::DoNotCull,
                current_drawing_skybox: false,
            })
        }
    }

    pub fn use_program<M, const OUT: usize, T: Texture>(
        &mut self,
        program: &ShaderProgram<M, OUT, T>,
    ) {
        gl_call! {
            gl::UseProgram(program.id().to_primitive());
        }
    }

    fn cull_face_after_check(&mut self, cull_face: CullFace) {
        if cull_face == self.current_cull_face {
            return;
        }
        self.current_cull_face = cull_face;

        let cull_enum = match cull_face {
            CullFace::DoNotCull => None,
            CullFace::FrontFace => Some(gl::FRONT),
            CullFace::BackFace => Some(gl::BACK),
        };

        match cull_enum {
            Some(value) => {
                gl_call! { gl::Enable(gl::CULL_FACE); }
                gl_call! { gl::CullFace(value); }
            }
            None => {
                gl_call! { gl::Disable(gl::CULL_FACE); }
            }
        }
    }

    pub fn force_cull_face(&mut self, cull_face: Option<CullFace>) {
        self.forced_cull_face = cull_face;
        if let Some(cull_face) = cull_face {
            self.cull_face_after_check(cull_face);
        }
    }

    pub fn cull_face(&mut self, cull_face: CullFace) -> Result<()> {
        match self.forced_cull_face {
            None => {
                self.cull_face_after_check(cull_face);

                Ok(())
            }
            Some(forced) => Err(Error::TriedToCullFaceWhenFaceCullingForcedByProgram {
                forced,
                attempted: cull_face,
            }
            .into()),
        }
    }

    pub fn drawing_skybox(&mut self, drawing_skybox: bool) {
        if drawing_skybox == self.current_drawing_skybox {
            return;
        }
        self.current_drawing_skybox = drawing_skybox;

        if drawing_skybox {
            gl_call! {
                gl::DepthMask(gl::FALSE);
            }
            gl_call! {
                gl::DepthFunc(gl::LEQUAL);
            }
        } else {
            gl_call! {
                gl::DepthMask(gl::TRUE);
            }
            gl_call! {
                gl::DepthFunc(gl::LESS);
            }
        }
    }
}
