use core::ops::{ControlFlow, FromResidual, Residual, Try};

use crate::framebuffer::FramebufferContext;
use crate::shader_program::ShaderProgramContext;
use crate::types::Query;

pub enum PollResult<D: Draw, E> {
    Draw(D),
    Error(E),
    Close,
    Restart,
}

pub enum PollResultResidual<E> {
    Close,
    Err(E),
    Restart,
}

impl<'a, E, D: Draw + 'a> Residual<D> for PollResultResidual<E> {
    type TryType = PollResult<D, E>;
}

impl<E, D: Draw> FromResidual<Result<!, E>> for PollResult<D, E> {
    fn from_residual(residual: Result<!, E>) -> Self {
        let e = residual.into_err();
        Self::Error(e)
    }
}

impl<E, D: Draw> Try for PollResult<D, E> {
    type Output = D;
    type Residual = PollResultResidual<E>;

    fn branch(self) -> core::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Error(err) => ControlFlow::Break(PollResultResidual::Err(err)),
            Self::Close => ControlFlow::Break(PollResultResidual::Close),
            Self::Draw(draw) => ControlFlow::Continue(draw),
            Self::Restart => ControlFlow::Break(PollResultResidual::Restart),
        }
    }

    fn from_output(output: Self::Output) -> Self {
        Self::Draw(output)
    }
}

impl<E, D: Draw> FromResidual<PollResultResidual<E>> for PollResult<D, E> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            PollResultResidual::Err(err) => Self::Error(err),
            PollResultResidual::Close => Self::Close,
            PollResultResidual::Restart => Self::Restart,
        }
    }
}

pub trait Draw {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut;

    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut
    where
        Self: Sized;
}

#[must_use]
enum DrawOutInner {
    Draw,
    QueryOutput { name: usize, query_object: Query },
    MultiQueryOutput { values: Vec<(usize, Query)> },
}

pub struct DrawOut {
    inner: DrawOutInner,
}

impl DrawOut {
    pub const OK: Self = Self {
        inner: DrawOutInner::Draw,
    };

    pub(crate) fn make_query(name: usize, query_object: Query) -> Self {
        Self {
            inner: DrawOutInner::QueryOutput { name, query_object },
        }
    }

    pub(crate) fn get_query(self) -> Option<Vec<(usize, Query)>> {
        match self.inner {
            DrawOutInner::Draw => None,
            DrawOutInner::QueryOutput { name, query_object } => Some(vec![(name, query_object)]),
            DrawOutInner::MultiQueryOutput { values } => Some(values),
        }
    }
}

// impl<'a, T: Draw + ?Sized + 'a> Draw for Box<T> {
// fn draw(
// self: Box<Self>,
// fb_context: &mut FramebufferContext,
// sp_context: &mut ShaderProgramContext,
// ) -> DrawOut {
// let new_self = self as Box<dyn Draw>;
// new_self.draw(fb_context, sp_context)
// }
//
// fn draw_sized(
// self,
// fb_context: &mut FramebufferContext,
// sp_context: &mut ShaderProgramContext,
// ) -> DrawOut
// where
// Self: Sized,
// {
// self.draw(fb_context, sp_context)
// }
// }

impl<'a> Draw for Box<dyn Draw + 'a> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let inner = *self;
        inner.draw(fb_context, sp_context)
    }

    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut
    where
        Self: Sized,
    {
        let inner = self as Box<dyn Draw>;
        inner.draw(fb_context, sp_context)
    }
}

impl<'a, T: Draw + 'a> Draw for [T] {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        shader_marker: &mut ShaderProgramContext,
    ) -> DrawOut {
        let mut outputs = Vec::new();
        for group in self {
            match group.draw_sized(fb_context, shader_marker).inner {
                DrawOutInner::QueryOutput {
                    name, query_object, ..
                } => {
                    outputs.push((name, query_object));
                }
                DrawOutInner::MultiQueryOutput { values, .. } => {
                    outputs.extend(values);
                }
                _ => {}
            }
        }

        if outputs.is_empty() {
            DrawOut::OK
        } else {
            DrawOut {
                inner: DrawOutInner::MultiQueryOutput { values: outputs },
            }
        }
    }
}

impl<'a, T: Draw + 'a> Draw for Vec<T> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let slice = self.into_boxed_slice();
        slice.draw(fb_context, sp_context)
    }

    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut
    where
        Self: Sized,
    {
        let slice = self.into_boxed_slice();
        slice.draw(fb_context, sp_context)
    }
}

#[derive(Debug, Default)]
pub struct DrawNothing;

impl DrawNothing {
    pub fn new() -> Self {
        DrawNothing
    }
}

impl Draw for DrawNothing {
    fn draw(self: Box<Self>, _: &mut FramebufferContext, _: &mut ShaderProgramContext) -> DrawOut {
        DrawOut::OK
    }

    fn draw_sized(self, _: &mut FramebufferContext, _: &mut ShaderProgramContext) -> DrawOut
    where
        Self: Sized,
    {
        DrawOut::OK
    }
}
