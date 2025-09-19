use std::iter::Sum;
use std::ops::{Add, Div, Mul};

use graphics::linear_algebra::{UnitVector, Vector};

use crate::modelling::cubic::geometry::{Orientation, Pose};
use crate::modelling::{BoundingBox, Cubic, SimpleVertex};

#[derive(Debug)]
pub struct Entity {
    pub model: Cubic,
    bounding_box: BoundingBox<SimpleVertex, 3>,
    pub velocity: Vector<3>,
}

impl Entity {
    pub fn from_cubic(model: Cubic, depth: usize) -> Option<Self> {
        let bounding_box = BoundingBox::from_cubic(&model, depth)?;

        Some(Self {
            model,
            bounding_box,
            velocity: Default::default(),
        })
    }

    pub fn get_cubic_mut(&mut self) -> &mut Cubic {
        &mut self.model
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

    pub fn bounce_off(&mut self, other: &mut Self) {
        if let Some((lhs, rhs)) = self.detect_collision(other).and_then(|mut vec| vec.pop()) {
            let lhs_norm = get_average_normal(lhs);
            let rhs_norm = get_average_normal(rhs);

            // lhs entity is reflected about rhs norm;
            // rhs entity is reflected about lhs norm;
            self.velocity = self.velocity.reflect(rhs_norm);
            other.velocity = other.velocity.reflect(lhs_norm);
        }
    }

    pub fn update(&mut self, timestep: f32) {
        let delta_pose = Pose::new_from_orientation_translation(
            Orientation::default(),
            self.velocity.scale(timestep),
        );
        let new_pose = self.model.root().apply_after(delta_pose);
        *self.model.root_mut() = new_pose;
    }
}

fn get_average_normal(input: Vec<[SimpleVertex; 3]>) -> UnitVector<3> {
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
