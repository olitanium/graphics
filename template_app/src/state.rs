use std::collections::HashSet;
use std::time::Duration;

use engine::types::TexDim;
use engine::{ApiState, DrawNothing, Event};

pub struct State {}

impl ApiState for State {
    type DrawOutput<'a>
        = DrawNothing
    where
        Self: 'a;
    type Error = ();

    fn new(_initial_size: (TexDim, TexDim)) -> Result<Self, Self::Error> {
        Ok(Self {})
    }

    fn poll<'a>(
        &'a mut self,
        events: Vec<engine::Event<'a>>,
    ) -> engine::PollResult<Self::DrawOutput<'a>, Self::Error> {
        let mut opt_default = None;
        let mut frame_time = Duration::default();
        let mut keys = HashSet::default();

        for event in events {
            match event {
                Event::FrameTime(in_ft) => frame_time = in_ft,
                Event::WindowBuffer(in_buff) => opt_default = Some(in_buff),
                Event::Keyboard { keys: in_keys, .. } => keys = in_keys,
                _ => {}
            }
        }

        engine::PollResult::Draw(DrawNothing)
    }
}
