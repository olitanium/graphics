use infinite_window::InfiniteWindowIter;

// use super::pose::PoseLerp;
use super::{Pose, Sclerp, YieldsPose};

#[derive(Debug, Clone)]
pub struct Animation {
    // time is the cumulative START TIME of the keyframe.
    keyframes: Vec<(f32 /* time */, Pose)>,
    duration: Option<f32>,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            keyframes: Vec::default(),
            duration: Some(0.0),
        }
    }
}

impl Animation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, pose: Pose, duration: f32) -> Result<(), Pose> {
        if let Some(total_duration) = self.duration.as_mut() {
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
        self.duration = None;
        self.keyframes.clear();
        self.keyframes.push((0.0, pose));
    }
}

impl Extend<(f32, Pose)> for Animation {
    fn extend<T: IntoIterator<Item = (f32, Pose)>>(&mut self, iter: T) {
        for (time, pose) in iter {
            let _ = self.push(pose, time);
        }
    }
}

impl YieldsPose for Animation {
    type Hint = f32;

    fn get_pose(&self, time: Self::Hint) -> Pose {
        match (self.keyframes.as_slice(), self.duration) {
            (&[], _) => Pose::default(),
            (&[(_, single)], _) => single,
            (slice, Some(duration)) => {
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

                let lerp = Sclerp::new(start_pose, end_pose, false, end_time - start_time);
                lerp.get_pose(local_time - start_time)
            }
            (slice @ &[.., (_, final_pose)], None) => slice
                .array_windows()
                .find(|&&[(start_time, _), (end_time, _)]| start_time <= time && time < end_time)
                .map_or(
                    final_pose,
                    |&[(start_time, start_pose), (end_time, end_pose)]| {
                        let lerp = Sclerp::new(start_pose, end_pose, false, end_time - start_time);
                        lerp.get_pose(time - start_time)
                    },
                ),
        }
    }
}
