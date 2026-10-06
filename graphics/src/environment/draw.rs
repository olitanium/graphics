use core::error::Error;
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

impl<E, D: Draw> Residual<D> for PollResultResidual<E> {
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

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
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
    ) -> Result<DrawOut, Box<dyn Error>>;

    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>>
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
    pub const OK: Result<Self, Box<dyn Error>> = Ok(Self {
        inner: DrawOutInner::Draw,
    });

    pub(crate) const fn make_query(name: usize, query_object: Query) -> Self {
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

impl Draw for Box<dyn Draw + '_> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let inner = *self;
        inner.draw(fb_context, sp_context)
    }

    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>>
    where
        Self: Sized,
    {
        let inner = self as Box<dyn Draw>;
        inner.draw(fb_context, sp_context)
    }
}

impl<T: Draw> Draw for [T] {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let mut outputs = Vec::new();
        for group in self {
            match group.draw_sized(fb_context, sp_context)?.inner {
                DrawOutInner::QueryOutput { name, query_object } => {
                    outputs.push((name, query_object));
                }
                DrawOutInner::MultiQueryOutput { values } => {
                    outputs.extend(values);
                }
                DrawOutInner::Draw => {}
            }
        }

        if outputs.is_empty() {
            DrawOut::OK
        } else {
            Ok(DrawOut {
                inner: DrawOutInner::MultiQueryOutput { values: outputs },
            })
        }
    }
}

impl<T: Draw> Draw for Vec<T> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let slice = self.into_boxed_slice();
        slice.draw(fb_context, sp_context)
    }

    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>>
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
    pub const fn new() -> Self {
        Self
    }
}

impl Draw for DrawNothing {
    fn draw(
        self: Box<Self>,
        _: &mut FramebufferContext,
        _: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        DrawOut::OK
    }

    fn draw_sized(
        self,
        _: &mut FramebufferContext,
        _: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>>
    where
        Self: Sized,
    {
        DrawOut::OK
    }
}
