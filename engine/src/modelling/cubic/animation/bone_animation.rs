use geometry::{Pose, PoseSclerp};
use infinite_window::InfiniteWindowIter;

#[derive(Debug, Clone)]
pub struct BoneAnimation {
    // time is the cumulative START TIME of the keyframe.
    pub(crate) keyframes: Vec<(f32 /* start time */, Pose)>,
    duration: Option<f32>,
}

impl Default for BoneAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl BoneAnimation {
    pub const fn new() -> Self {
        Self {
            keyframes: Vec::new(),
            duration: Some(0.0),
        }
    }

    pub fn push(&mut self, pose: Pose, duration: f32) -> Result<(), Pose> {
        if let Some(total_duration) = &mut self.duration {
            self.keyframes.push((*total_duration, pose));
            *total_duration += duration;
            Ok(())
        } else {
            Err(pose)
        }
    }

    pub fn push_final_non_repeat(&mut self, pose: Pose) -> Result<(), Pose> {
        if let Some(total_duration) = self.duration {
            self.keyframes.push((total_duration, pose));
            self.duration = None;
            Ok(())
        } else {
            Err(pose)
        }
    }

    pub fn set(&mut self, pose: Pose) {
        self.duration = Some(0.0);
        self.keyframes.clear();
        self.keyframes.push((0.0, pose));
    }
}

impl BoneAnimation {
    pub fn get_pose(&self, time: f32) -> Pose {
        match (self.keyframes.as_slice(), self.duration) {
            (&[], _) => Pose::default(),
            (&[(_, single)], _) => single,
            (slice, Some(duration)) => {
                #[expect(clippy::modulo_arithmetic, reason = "duration never negative")]
                let local_time = time % duration;
                let [&(start_time, start_pose), &(mut end_time, end_pose)] =
                    InfiniteWindowIter::new(slice)
                        .find(|&[&(start_time, _), &(end_time, _)]| {
                            let between = start_time <= local_time && local_time < end_time;

                            let is_in_overlap = end_time == 0.0;

                            between || is_in_overlap
                        })
                        .expect(
                            "infinitely repeating iterator must return a value or loop \
                             indefinitely",
                        );

                end_time = if end_time == 0.0 { duration } else { end_time };

                let sclerp = PoseSclerp::new(start_pose, end_pose, end_time - start_time, true);
                sclerp.get_pose(local_time - start_time)
            }
            (slice @ &[.., (_, final_pose)], None) => slice
                .array_windows()
                .find(|&&[(start_time, _), (end_time, _)]| start_time <= time && time < end_time)
                .map_or(
                    final_pose,
                    |&[(start_time, start_pose), (end_time, end_pose)]| {
                        let sclerp =
                            PoseSclerp::new(start_pose, end_pose, end_time - start_time, true);
                        sclerp.get_pose(time - start_time)
                    },
                ),
        }
    }
}
