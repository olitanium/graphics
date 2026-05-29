use std::sync::Mutex;

use utils::error_boilerplate;

use super::{CullFace, ShaderProgram};
use crate::texture::Texture;
use crate::types::ShaderProgramId;
use crate::{Model, gl_call};

#[derive(Debug)]
pub struct ShaderProgramContext {
    current_program: Option<ShaderProgramId::Primitive>,
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
                current_program: None,
                forced_cull_face: None,
                current_cull_face: CullFace::DoNotCull,
                current_drawing_skybox: false,
            })
        }
    }

    pub fn use_program<M: Model, O, const OUT: usize, T: Texture>(
        &mut self,
        program: &ShaderProgram<M, O, OUT, T>,
    ) {
        if let Some(curr_program_id) = self.current_program
            && curr_program_id == program.id().as_primitive()
        {
            // intentionally blank, do nothing if this is already the same
            // program
        } else {
            gl_call! {
                gl::UseProgram(program.id().as_primitive());
            }

            self.force_cull_face(program.force_cull_face);
            self.current_program = Some(program.id().as_primitive())
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
}

#[derive(Debug)]
#[non_exhaustive]
pub enum FaceCullingForcedByProgramError {
    Forced(CullFace),
}
error_boilerplate!(FaceCullingForcedByProgramError);

impl ShaderProgramContext {
    pub fn cull_face(
        &mut self,
        cull_face: CullFace,
    ) -> Result<(), FaceCullingForcedByProgramError> {
        match self.forced_cull_face {
            None => {
                self.cull_face_after_check(cull_face);

                Ok(())
            }
            Some(forced) => Err(FaceCullingForcedByProgramError::Forced(forced)),
        }
    }
}

impl ShaderProgramContext {
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
