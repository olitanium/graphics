use engine::linear_algebra::Vector;
use engine::modelling::cubic::geometry::AggregateCalcAnim;

use crate::state::State;

impl State {
    pub fn physics(&mut self, _frame_time: f32) -> AggregateCalcAnim {
        let time = self.time;

        let light_pos = Vector::new([
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).cos(),
        ]);

        if let Some(light) = self.light_group.point.get_mut(0) {
            light.light.position = light_pos;
        }

        if let Some(light) = self.light_group.spot.get_mut(0) {
            light.light.direction = self.camera.direction(&());
            light.light.position = self.camera.position(&());
        }

        let morgan = self.morgan.take();
        let mut opt_morgan = morgan.map(|morg| {
            let pose = morg.inner.pose;
            (morg, pose)
        });
        let mut none = None;

        let grappling_hook_calc_anim = self
            .grappling_hook
            .update(_frame_time, if false { &mut none } else { &mut opt_morgan });

        self.morgan = opt_morgan.map(|(morgan, _)| morgan).take();

        grappling_hook_calc_anim
    }
}
