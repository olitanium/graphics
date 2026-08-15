use geometry::Pose;
use triangle_intersection::Vector;

use crate::entity::Mass;

#[derive(Debug, Default)]
pub struct SimplePhysics {
    pose: Pose,
    velocity: Vector<3>,
    mass: Mass,
    collision_force: Vector<3>,
}

impl SimplePhysics {
    pub const fn pose(&self) -> &Pose {
        &self.pose
    }

    pub const fn velocity(&self) -> &Vector<3> {
        &self.velocity
    }

    pub const fn mass(&self) -> &Mass {
        &self.mass
    }

    pub const fn set_mass(&mut self, mass: Mass) {
        self.mass = mass;
    }

    pub const fn set_pose_velocity(&mut self, pose: Pose, velocity: Vector<3>) {
        self.pose = pose;
        self.set_velocity(velocity);
    }

    pub const fn set_velocity(&mut self, velocity: Vector<3>) {
        self.velocity = velocity;
    }

    pub fn process<F>(&mut self, timestep: f32, mut external_force: F) -> &Pose
    where
        F: FnMut(
            Vector<3>, // velocity
            Vector<3>, // position
        ) -> Vector<3>,
    {
        let force = external_force(self.velocity, self.pose.translation()) + self.collision_force;
        self.collision_force = Vector::ZERO;

        let acceleration = force * self.mass.inverse_mass();

        self.pose
            .push_change(Pose::new_from_translation(self.velocity * timestep));
        self.velocity += acceleration * timestep;

        &self.pose
    }
}
