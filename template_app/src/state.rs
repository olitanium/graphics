use std::collections::HashSet;

use engine::framebuffer::DefaultFramebuffer;
use engine::modelling::cubic::{Camera, SimpleCubic};
use engine::{ApiState, DrawNothing, Event, SimpleEntity};

pub struct State {}

impl ApiState for State {
    type Error = ();

    fn new(
        initial_size: (engine::types::TexDim, engine::types::TexDim),
    ) -> Result<Self, Self::Error> {
        Ok(Self {})
    }

    fn poll<'a>(
        &'a mut self,
        events: Vec<engine::Event<'a>>,
    ) -> engine::PollResult<'a, Self::Error> {
        let mut opt_default = None;
        let mut frame_time = Default::default();
        let mut keys = Default::default();

        for event in events {
            match event {
                Event::FrameTime(in_ft) => frame_time = in_ft,
                Event::WindowBuffer(in_buff) => opt_default = Some(in_buff),
                Event::Keyboard {
                    keys: in_keys,
                    buffer: _,
                } => keys = in_keys,
                _ => {}
            }
        }

        engine::PollResult::Draw(DrawNothing::new())
    }
}
