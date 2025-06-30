use super::quaternion::UnitQuaternion;
use crate::linear_algebra::{Matrix, UnitVector, Vector};
use crate::modelling::cubic::geometry::orientation::quaternion::Quaternion;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Orientation {
    quaternion: UnitQuaternion,
}

impl Orientation {
    pub fn into_quaternion(self) -> UnitQuaternion {
        self.quaternion
    }

    pub fn from_quaternion(quaternion: UnitQuaternion) -> Self {
        Self { quaternion }
    }

    pub fn act_on(self, vector: Vector<3>) -> Vector<3> {
        let vec_as_quarternion = Quaternion::from_vec(vector);
        let output_quaternion =
            self.quaternion.q() * vec_as_quarternion * self.quaternion.q().conjugate();
        output_quaternion.vector()
    }

    pub fn combine(self, after: Orientation) -> Orientation {
        let new_quaternion = (after.quaternion.q() * self.quaternion.q()).normalize();
        Orientation::from_quaternion(new_quaternion)
    }

    pub fn axis_angle(direction: UnitVector<3>, angle: f32) -> Orientation {
        let unit_quaternion = Quaternion::axis_angle(angle, direction);
        Orientation::from_quaternion(unit_quaternion)
    }

    pub fn new_forward_up(forward: UnitVector<3>, global_up: UnitVector<3>) -> Self {
        let left = global_up.v().cross(forward.v()).normalize();
        let view_up = forward.v().cross(left.v()).normalize();

        let matrix = Matrix::from_col_major([
            left.v(),    // left = x
            view_up.v(), // up = y
            forward.v(), // forward = z
        ]);

        let quaternion = UnitQuaternion::from_matrix(matrix);
        Self { quaternion }
    }

    pub fn view_forward(&self) -> UnitVector<3> {
        let w = self.quaternion.q().scalar();
        let [x, y, z] = self.quaternion.q().vector().into_inner();

        Vector::new([
            2.0 * (x * z + w * y),
            2.0 * (y * z - w * x),
            1.0 - 2.0 * (x * x + y * y),
        ])
        .normalize() // z
    }

    pub fn view_up(&self) -> UnitVector<3> {
        let w = self.quaternion.q().scalar();
        let [x, y, z] = self.quaternion.q().vector().into_inner();

        Vector::new([
            2.0 * (x * y - w * z),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z + w * x),
        ])
        .normalize() // z
    }

    pub fn view_left(&self) -> UnitVector<3> {
        let w = self.quaternion.q().scalar();
        let [x, y, z] = self.quaternion.q().vector().into_inner();

        Vector::new([
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y + w * z),
            2.0 * (x * z - w * y),
        ])
        .normalize() // z
    }

    pub fn look_left(&mut self, angle: f32) {
        let effect = Quaternion::axis_angle(angle, UnitVector::new_unchecked([0.0, 1.0, 0.0]));
        self.quaternion = (self.quaternion.q() * effect.q()).normalize();
    }

    pub fn look_up(&mut self, angle: f32) {
        let effect = Quaternion::axis_angle(angle, Vector::new([-1.0, 0.0, 0.0]).normalize());
        self.quaternion = (self.quaternion.q() * effect.q()).normalize();
    }

    pub fn roll_ccw(&mut self, angle: f32) {
        let effect = Quaternion::axis_angle(angle, UnitVector::new_unchecked([0.0, 0.0, -1.0]));
        self.quaternion = (self.quaternion.q() * effect.q()).normalize();
    }

    pub fn reverse_direction(&mut self) {
        let rotation = Quaternion::axis_angle(
            std::f32::consts::PI,
            UnitVector::new_unchecked([0.0, 1.0, 0.0]),
        );
        self.quaternion = (self.quaternion.q() * rotation.q()).normalize()
    }

    pub fn new_from_to(from: Vector<3>, to: Vector<3>, global_up: UnitVector<3>) -> Self
    where
        Self: Sized,
    {
        let forward = (to - from).normalize();
        Self::new_forward_up(forward, global_up)
    }

    pub fn as_matrix(&self) -> Matrix<4, 4> {
        // forward
        // up
        // right => -left
        // Matrix::from_col_major([
        // self.view_forward().v().truncate(),
        // self.view_up().v().truncate(),
        // -self.view_left().v().truncate(),
        // Vector::from([0.0, 0.0, 0.0, 1.0]),
        // ])

        Matrix::from_col_major([
            self.view_left().v().truncate(),
            self.view_up().v().truncate(),
            self.view_forward().v().truncate(),
            Vector::from([0.0, 0.0, 0.0, 1.0]),
        ])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_values() {
        let orient = Orientation::new_from_to(
            Vector::from([0.0, 0.0, 0.0]),
            Vector::from([1.0, 1.0, 1.0]),
            Vector::from([0.0, 1.0, 0.0]).normalize(),
        );
        println!("matrix: {:?}", orient.as_matrix());

        println!("orientation: {:?}", orient.into_quaternion());
    }

    #[test]
    fn test_y() {
        fn is_similar_enough(one: Vector<3>, two: Vector<3>) -> bool {
            let diff = one - two;
            diff.into_iter().all(|x| x.abs() < 0.001)
        }

        let tests = [
            (
                Vector::new([0.0, 0.0, 1.0]).normalize(),
                Vector::new([0.0, 1.0, 0.0]).normalize(),
            ),
            (
                Vector::new([1.0, 0.0, 0.0]).normalize(),
                Vector::new([0.0, 1.0, 0.0]).normalize(),
            ),
            (
                Vector::new([0.0, 1.0, 0.0]).normalize(),
                Vector::new([0.1, 0.0, 0.0]).normalize(),
            ),
            (
                Vector::new([0.0, 1.0, 0.0]).normalize(),
                Vector::new([0.0, 0.0, 1.0]).normalize(),
            ),
            (
                Vector::new([0.0, 0.0, 1.0]).normalize(),
                Vector::new([1.0, 0.0, 0.0]).normalize(),
            ),
        ];

        for (up, direction) in tests {
            let orientation = Orientation::new_forward_up(direction, up);

            let should_be_up = orientation.act_on(Vector::new([0.0, 1.0, 0.0]));
            let should_be_direction = orientation.act_on(Vector::new([0.0, 0.0, 1.0]));

            assert!(is_similar_enough(up.v(), should_be_up.normalize().v()));
            assert!(is_similar_enough(
                direction.v(),
                should_be_direction.normalize().v()
            ));
        }
    }
}
