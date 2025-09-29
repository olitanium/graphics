use graphics::linear_algebra::{UnitVector, Vector};

use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{BoundingBox, Cubic, SimpleVertex};

#[derive(Debug)]
pub struct Entity {
    pub model: Cubic,
    pub mass: f32,
    bounding_box: BoundingBox<SimpleVertex, 3>,
    pub velocity: Vector<3>,
}

impl Entity {
    pub fn from_cubic(model: Cubic, depth: usize, mass: f32) -> Option<Self> {
        let bounding_box = BoundingBox::from_cubic(&model, depth)?;

        Some(Self {
            model,
            mass,
            bounding_box,
            velocity: Default::default(),
        })
    }

    pub fn get_bounding_box(&self) -> &BoundingBox<SimpleVertex, 3> {
        &self.bounding_box
    }

    pub fn detect_collision(
        &self,
        other: &Self,
    ) -> Option<Vec<(Vec<[SimpleVertex; 3]>, Vec<[SimpleVertex; 3]>)>> {
        self.bounding_box.detect_collision(
            self.model.root(),
            &other.bounding_box,
            other.model.root(),
        )
    }

    pub fn elastic_collision_no_rotation(&mut self, other: &mut Self) -> bool {
        if let Some(vec) = self.detect_collision(other) {
            // println!("vec: {vec:#?}");
            let (lhs_manyvec, rhs_manyvec): (Vec<Vec<_>>, Vec<Vec<_>>) = vec.into_iter().unzip();

            let lhs = lhs_manyvec.into_iter().flatten();
            let rhs = rhs_manyvec.into_iter().flatten();

            let lhs_norm = get_average_normal(lhs);
            let rhs_norm = get_average_normal(rhs);

            let average = (lhs_norm.v() - rhs_norm.v()).normalize();

            // lhs entity is reflected about rhs norm
            // rhs entity is reflected about lhs norm
            let self_change = 2.0 / (1.0 + other.mass / self.mass)
                * average.v().dot(other.velocity - self.velocity);
            let other_change = 2.0 / (1.0 + self.mass / other.mass)
                * average.v().dot(self.velocity - other.velocity);

            self.velocity = self.velocity + average.v() * self_change; //.reflect(-average);
            other.velocity = other.velocity + average.v() * other_change; //.reflect(average);

            // TODO: make neater without magic SMALL_TIMESTEP
            const SMALL_TIMESTEP: f32 = 0.05;
            let other_push = Pose::new_from_translation(other.velocity.scale(SMALL_TIMESTEP));
            let self_push = Pose::new_from_translation(self.velocity.scale(SMALL_TIMESTEP));
            other.model.root_mut().push_change(other_push);
            self.model.root_mut().push_change(self_push);
            while let Some(_) = self.detect_collision(other) {
                other.model.root_mut().push_change(other_push);
                self.model.root_mut().push_change(self_push);
            }

            true
        } else {
            false
        }
    }

    pub fn update(&mut self, timestep: f32) {
        let delta_pose = Pose::new_from_translation(self.velocity.scale(timestep));
        self.model.root_mut().push_change(delta_pose);
    }
}

fn get_average_normal<I: IntoIterator<Item = [SimpleVertex; 3]>>(input: I) -> UnitVector<3> {
    let all_normals = input
        .into_iter()
        .map(|triangle| triangle.map(|vertex| vertex.normal.v()))
        .flatten()
        .collect::<Vec<_>>();

    let len = all_normals.len();
    let one_len = 1.0 / len as f32;

    let sum: Vector<3> = all_normals.into_iter().sum();
    sum.scale(one_len).normalize()
}
