use crate::{
    matrix::Matrix,
    vector::{UnitVector, Vector},
};

#[derive(Debug, Clone, Copy)]
enum Up {
    Relative(UnitVector<3>),
    Fixed { up: UnitVector<3>, cos_no_cone: f32 },
}

impl Up {
    fn inner(self) -> UnitVector<3> {
        match self {
            Self::Relative(up) | Self::Fixed { up, .. } => up,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Orientation {
    forward: UnitVector<3>,
    up: Up,
}

impl Default for Orientation {
    #[inline]
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Orientation {
    #[inline]
    pub fn builder() -> Builder {
        Builder::default()
    }

    #[inline]
    pub fn right(self) -> Vector<3> {
        self.forward().cross(self.up.inner().into()).normalize()
    }

    #[inline]
    pub fn as_matrix(self, right: Option<Vector<3>>) -> Matrix<4, 4> {
        let right = right.unwrap_or_else(|| self.right());

        let mut out = Matrix::from_col_major([
            self.forward().truncate(),
            self.view_up(Some(right)).truncate(),
            right.truncate(),
            Vector::new_zero(),
        ]);
        out[(3, 3)] = 1.0;

        out
    }

    #[inline]
    pub fn forward(self) -> Vector<3> {
        self.forward.into()
    }

    #[inline]
    pub fn view_up(self, right: Option<Vector<3>>) -> Vector<3> {
        match self.up {
            Up::Relative(up) => up.into(),
            Up::Fixed {
                up: _,
                cos_no_cone: _,
            } => Vector::cross(right.unwrap_or_else(|| self.right()), self.forward()).normalize(),
        }
    }

    pub fn real_up(self) -> Vector<3> {
        self.up.inner().into()
    }

    #[inline]
    pub fn look_up(&mut self, angle: f32) {
        let old_forward = self.forward();
        match self.up {
            Up::Relative(up) => {
                let forward = old_forward + Vector::from(up).scale(angle);
                self.forward = forward.into();

                let projection = Vector::dot(up.as_ref(), &self.forward());
                self.up =
                    Up::Relative((Vector::from(up) - self.forward().scale(projection)).into());
            }
            Up::Fixed { up, cos_no_cone } => {
                let fdotu = Vector::dot(&old_forward, up.as_ref());
                if !(angle > 0.0 && fdotu > cos_no_cone || angle < 0.0 && fdotu < -cos_no_cone) {
                    let camera_up = self.view_up(None);
                    self.forward = (old_forward + camera_up.scale(angle)).into();
                }
            }
        }
    }

    #[inline]
    pub fn look_right(&mut self, angle: f32) {
        self.forward = (self.forward() + self.right().scale(angle)).into();
    }

    #[inline]
    pub fn roll_clockwise(&mut self, angle: f32) {
        let right = self.right();
        let view_up = self.view_up(Some(right));

        if let Up::Relative(up) = &mut self.up {
            *up = (view_up + right.scale(angle)).into();
        }
    }

    #[inline]
    pub fn reverse_direction(&mut self) {
        self.forward = self.forward().flip().into();
    }

    #[inline]
    pub fn forward_motion_orientation(mut self) -> Self {
        self.forward = self.forward_motion_direction().into();
        self
    }

    #[inline]
    pub fn forward_motion_direction(self) -> Vector<3> {
        match self.up {
            Up::Fixed { up, cos_no_cone: _ } => Vector::from(up)
                .cross(self.forward().cross(up.into()))
                .normalize(),
            Up::Relative(_) => self.forward(),
        }
    }
}

#[derive(Debug)]
enum BuilderUp {
    Relative(UnitVector<3>),
    Fixed(UnitVector<3>),
}

#[derive(Default, Debug)]
pub struct Builder {
    forward: Option<UnitVector<3>>,
    up: Option<BuilderUp>,
    no_cone: Option<f32>,
}

impl Builder {
    #[inline]
    pub fn looking_at(mut self, centre: Vector<3>, target: Vector<3>) -> Self {
        self.forward = Some(Vector::from_to(centre, target).into());
        self
    }

    #[inline]
    pub fn forward(mut self, forward: Vector<3>) -> Self {
        self.forward = Some(forward.into());
        self
    }

    #[inline]
    pub fn relative_up<V: Into<UnitVector<3>>>(mut self, up: V) -> Self {
        self.up = Some(BuilderUp::Relative(up.into()));
        self
    }

    #[inline]
    pub fn fixed_up<V: Into<UnitVector<3>>>(mut self, up: V) -> Self {
        self.up = Some(BuilderUp::Fixed(up.into()));
        self
    }

    #[inline]
    pub fn no_cone_degrees(mut self, degrees: f32) -> Self {
        self.no_cone = Some(degrees.to_radians());
        self
    }

    #[inline]
    pub fn no_cone_radians(mut self, radians: f32) -> Self {
        self.no_cone = Some(radians);
        self
    }

    #[inline]
    pub fn build(self) -> Orientation {
        let forward = self
            .forward
            .unwrap_or_else(|| unsafe { Vector::new_normal_unchecked([1.0, 0.0, 0.0]) }.into());
        let cos_no_cone = self.no_cone.map_or(0.99, f32::cos);
        let up = match self.up {
            Some(BuilderUp::Fixed(up)) => Up::Fixed { up, cos_no_cone },
            Some(BuilderUp::Relative(up)) => Up::Relative(up),
            None => Up::Fixed {
                up: unsafe { Vector::new_normal_unchecked([0.0, 1.0, 0.0]) }.into(),
                cos_no_cone,
            },
        };

        Orientation { forward, up }
    }
}
