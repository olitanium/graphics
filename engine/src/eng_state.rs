use graphics::types::TexDim;
use graphics::{ApiResult, Draw, Event};

use crate::{EngError, EngResult, VecDraw};

pub trait EngState: Sized {
    type Error: core::error::Error;

    fn new(initial_size: (TexDim, TexDim)) -> EngResult<Self, Self::Error>;

    fn poll<'a>(&'a mut self, events: Vec<Event<'a>>) -> EngResult<VecDraw<'a>, Self::Error>;
}

#[macro_export]
macro_rules! derive_api_state {
    ($slf:path) => {
        impl ApiState for $slf {
            type Error = EngError<<$slf as EngState>::Error>;

            fn new(initial_size: (TexDim, TexDim)) -> $crate::ApiResult<Self, Self::Error> {
                let out = <$slf as EngState>::new(initial_size)?;
                Ok(out)
            }

            fn poll<'a>(
                &'a mut self,
                events: Vec<Event<'a>>,
            ) -> $crate::ApiResult<Vec<Box<dyn $crate::Draw + 'a>>, Self::Error> {
                let out = <$slf as EngState>::poll(self, events)?;
                Ok(out)
            }
        }
    };
}

// use graphics::ApiState;
// impl<T: EngState> ApiState for T {
// type Error = EngError<<Self as EngState>::Error>;
//
// fn new(initial_size: (TexDim, TexDim)) -> ApiResult<Self, Self::Error> {
// let out = <Self as EngState>::new(initial_size)?;
// Ok(out)
// }
//
// fn poll<'a>(
// &'a mut self,
// events: Vec<Event<'a>>,
// ) -> ApiResult<Vec<Box<dyn Draw + 'a>>, Self::Error> {
// let out = <Self as EngState>::poll(self, events)?;
// Ok(out)
// }
// }
