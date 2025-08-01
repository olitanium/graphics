use builder::MissingPose;
use graphics::linear_algebra::{Matrix, UnitVector, Vector};

use super::geometry::{Orientation, YieldsPose};

mod builder;
mod projection;
pub use projection::Projection;

mod orientation;
pub use builder::Builder;
pub use orientation::CameraPose;

#[derive(Clone, Debug)]
pub struct Camera<P: YieldsPose> {
    pub pose: P,
    pub radius: f32, // first or third-person
    pub projection: Projection,
}

pub fn builder() -> Builder<MissingPose> {
    Builder::new()
}

impl<P: YieldsPose> Camera<P> {
    pub fn centre(&self, hint: P::Hint) -> Vector<3> {
        self.pose.get_position(hint)
    }

    pub fn orientation(&self, hint: P::Hint) -> Orientation {
        self.pose.get_orientation(hint)
    }

    pub fn first_person(&self) -> bool {
        self.radius < 0.01
    }

    pub fn position(&self, hint: P::Hint) -> Vector<3> {
        if self.first_person() {
            self.centre(hint)
        } else {
            let pose = self.pose.get_pose(hint);
            pose.translation() - pose.orientation().view_forward().v().scale(self.radius)
        }
    }

    pub fn direction(&self, hint: P::Hint) -> UnitVector<3> {
        self.orientation(hint).view_forward()
    }

    pub fn radius_out(&mut self, distance: f32) -> f32 {
        self.radius += distance;
        if self.radius < 0.0 {
            self.radius = 0.0;
        }
        self.radius
    }

    pub fn look_at(&self, hint: P::Hint) -> Matrix<4, 4> {
        self.projection.as_matrix() * self.view(hint)
    }

    pub fn view(&self, hint: P::Hint) -> Matrix<4, 4> {
        let pose = self.pose.get_pose(hint);

        let camera_right = -pose.orientation().view_left();
        let camera_up = pose.orientation().view_up();
        let camera_backwards = -pose.orientation().view_forward();

        #[rustfmt::skip]
        let lhs = Matrix::from_col_major([
            [ camera_right[0], camera_up[0], camera_backwards[0], 0.0,     ],
            [ camera_right[1], camera_up[1], camera_backwards[1], 0.0,     ],
            [ camera_right[2], camera_up[2], camera_backwards[2], 0.0,     ],
            [             0.0,          0.0,                 0.0, 1.0_f32, ],
        ]);

        let rhs = Matrix::transform_translate(-pose.translation());

        lhs * rhs
    }
}
