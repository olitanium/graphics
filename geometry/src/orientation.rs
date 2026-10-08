use core::f32;

use linear_algebra::{Matrix, UnitVector, Vector, vector};
use quaternion::{Quaternion, UnitQuaternion};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Orientation {
    quaternion: UnitQuaternion,
}

impl Orientation {
    pub const IDENTITY: Self = Self::from_unitquaternion(UnitQuaternion::IDENTITY);

    #[must_use]
    pub const fn into_quaternion(self) -> UnitQuaternion {
        self.quaternion
    }

    #[must_use]
    pub const fn from_unitquaternion(quaternion: UnitQuaternion) -> Self {
        Self { quaternion }
    }

    #[must_use]
    pub const fn act_on(self, vector: Vector<3>) -> Vector<3> {
        let vec_as_quarternion = Quaternion::from_vector(vector);
        let output_quaternion =
            self.quaternion.q() * vec_as_quarternion * self.quaternion.q().conjugate();
        output_quaternion.vector()
    }

    #[must_use]
    pub const fn act_on_unit(self, unit: UnitVector<3>) -> UnitVector<3> {
        UnitVector::new_unchecked(self.act_on(unit.v()).into_inner())
    }

    #[must_use]
    pub const fn combine(self, after: Self) -> Self {
        let new_quaternion = (after.quaternion.q() * self.quaternion.q()).normalize();
        Self::from_unitquaternion(new_quaternion)
    }

    #[must_use]
    pub fn axis_angle(direction: UnitVector<3>, angle: f32) -> Self {
        let unit_quaternion = Quaternion::axis_angle(angle, direction);
        Self::from_unitquaternion(unit_quaternion)
    }

    #[must_use]
    pub const fn new_forward_up(forward: UnitVector<3>, global_up: UnitVector<3>) -> Option<Self> {
        let left = global_up.v().cross(forward.v()).try_normalize()?;
        let view_up = forward.v().cross(left.v()).try_normalize()?;

        let matrix = Matrix::from_col_major([
            left.into_inner(),    // left = x
            view_up.into_inner(), // up = y
            forward.into_inner(), // forward = z
        ]);

        Some(Self::from_matrix(matrix))
    }

    #[must_use]
    pub fn new_forward_up_try_two(
        forward: UnitVector<3>,
        global_up: UnitVector<3>,
        up_two: UnitVector<3>,
    ) -> Option<Self> {
        debug_assert_ne!(
            global_up, up_two,
            "`global_up` and `up_two` are the same at {global_up:?}"
        );
        // should never fail in release because the `Self::new_forward_up` only
        // fails when forward == up;
        Self::new_forward_up(forward, global_up).or_else(|| Self::new_forward_up(forward, up_two))
    }

    #[must_use]
    pub const fn from_matrix(matrix: Matrix<3, 3>) -> Self {
        let quaternion = UnitQuaternion::from_matrix(matrix);
        Self { quaternion }
    }

    #[must_use]
    pub fn view_forward(&self) -> UnitVector<3> {
        let w = self.quaternion.q().scalar();
        let [x, y, z] = self.quaternion.q().vector().into_inner();

        vector![
            2.0 * w.mul_add(y, x * z),
            2.0 * w.mul_add(-x, y * z),
            2.0_f32.mul_add(-y.mul_add(y, x * x), 1.0),
        ]
        .try_normalize()
        .unwrap_or_default()
    }

    #[must_use]
    pub fn view_up(&self) -> UnitVector<3> {
        let w = self.quaternion.q().scalar();
        let [x, y, z] = self.quaternion.q().vector().into_inner();

        vector![
            2.0 * w.mul_add(-z, x * y),
            2.0_f32.mul_add(-z.mul_add(z, x * x), 1.0),
            2.0 * w.mul_add(x, y * z),
        ]
        .try_normalize()
        .unwrap_or_default() // y
    }

    #[must_use]
    pub fn view_left(&self) -> UnitVector<3> {
        let w = self.quaternion.q().scalar();
        let [x, y, z] = self.quaternion.q().vector().into_inner();

        vector![
            2.0_f32.mul_add(-z.mul_add(z, y * y), 1.0),
            2.0 * w.mul_add(z, x * y),
            2.0 * w.mul_add(-y, x * z),
        ]
        .try_normalize()
        .unwrap_or_default() // x
    }

    pub fn look_left(&mut self, angle: f32) {
        let effect = Quaternion::axis_angle(angle, UnitVector::new_unchecked([0.0, 1.0, 0.0]));
        self.quaternion = (self.quaternion.q() * effect.q()).normalize();
    }

    pub fn look_up(&mut self, angle: f32) {
        let effect = Quaternion::axis_angle(angle, -UnitVector::X);
        self.quaternion = (self.quaternion.q() * effect.q()).normalize();
    }

    pub fn roll_ccw(&mut self, angle: f32) {
        let effect = Quaternion::axis_angle(angle, UnitVector::new_unchecked([0.0, 0.0, -1.0]));
        self.quaternion = (self.quaternion.q() * effect.q()).normalize();
    }

    pub fn reverse_direction(&mut self) {
        let rotation =
            Quaternion::axis_angle(f32::consts::PI, UnitVector::new_unchecked([0.0, 1.0, 0.0]));
        self.quaternion = (self.quaternion.q() * rotation.q()).normalize();
    }

    #[must_use]
    pub const fn new_from_to(
        from: Vector<3>,
        to: Vector<3>,
        global_up: UnitVector<3>,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        let forward = (to - from).try_normalize()?;
        Self::new_forward_up(forward, global_up)
    }

    #[must_use]
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
            self.view_left().v().truncate().into_inner(),
            self.view_up().v().truncate().into_inner(),
            self.view_forward().v().truncate().into_inner(),
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

#[cfg(test)]
mod test {
    use linear_algebra::vector;

    use super::*;

    // #[test]
    // fn get_values() {
    // let orient =
    // Orientation::new_from_to(Vector::ZERO, Vector::from([1.0, 1.0, 1.0]),
    // UnitVector::Y) .unwrap();
    // println!("matrix: {:?}", orient.as_matrix());
    //
    // println!("orientation: {:?}", orient.into_quaternion());*/
    // }

    #[test]
    fn y() {
        fn is_similar_enough(one: Vector<3>, two: Vector<3>) -> bool {
            let diff = one - two;
            diff.into_iter().all(|x| x.abs() < 0.001)
        }

        let tests = [
            (UnitVector::Z, UnitVector::Y),
            (UnitVector::X, UnitVector::Y),
            (UnitVector::Y, UnitVector::X),
            (UnitVector::Y, UnitVector::Z),
            (UnitVector::Z, UnitVector::X),
        ];

        for (up, direction) in tests {
            let orientation = Orientation::new_forward_up(direction, up).unwrap();

            let should_be_up = orientation.act_on(vector![0.0, 1.0, 0.0]);
            let should_be_direction = orientation.act_on(vector![0.0, 0.0, 1.0]);

            assert!(is_similar_enough(up.v(), should_be_up));
            assert!(is_similar_enough(direction.v(), should_be_direction));
        }
    }
}
