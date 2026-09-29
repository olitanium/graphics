use core::time::Duration;
use std::collections::HashSet;

use engine::types::TexDim;
use engine::{ApiState, DrawNothing, Event, PollResult};

pub struct State;

impl ApiState for State {
    type DrawOutput<'a>
        = DrawNothing
    where
        Self: 'a;
    type Error = ();

    fn new(_initial_size: (TexDim, TexDim)) -> Result<Self, Self::Error> {
        Ok(Self {})
    }

    #[allow(clippy::allow_attributes, reason = "unused requires allow")]
    #[allow(unused, reason = "Template application")]
    #[expect(clippy::collection_is_never_read, reason = "Template App")]
    fn poll<'a>(
        &'a mut self,
        events: Vec<engine::Event<'a>>,
    ) -> PollResult<Self::DrawOutput<'a>, Self::Error> {
        let mut opt_default = None;
        let mut frame_time = Duration::default();
        let mut keys = HashSet::default();

        for event in events {
            match event {
                Event::FrameTime(in_ft) => frame_time = in_ft,
                Event::WindowBuffer(in_buff) => opt_default = Some(in_buff),
                Event::Keyboard { keys: in_keys, .. } => keys = in_keys,
                Event::ElapsedTime(_)
                | Event::WindowResize(_)
                | Event::Mouse { .. }
                | Event::Query(_)
                | _ => {}
            }
        }

        PollResult::Draw(DrawNothing)
    }
}
