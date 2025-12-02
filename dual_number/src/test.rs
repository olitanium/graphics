use crate::DualNumber;

#[test]
fn add() {
    let lhs = DualNumber::new(1, 2);
    let rhs = DualNumber::new(2, 3);

    assert_eq!(lhs + rhs, DualNumber::new(3, 5));
}

#[test]
fn mul() {
    let lhs = DualNumber::new(1, 2);
    let rhs = DualNumber::new(3, 5);

    assert_eq!(lhs * rhs, DualNumber::new(3, 11))
}

#[test]
fn mul_noncommut() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct NonCommutative(i32);

    impl std::ops::Mul for NonCommutative {
        type Output = Self;

        fn mul(self, rhs: Self) -> Self::Output {
            Self((self.0 + 1) * (rhs.0))
        }
    }

    impl std::ops::Add for NonCommutative {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(self.0 + rhs.0)
        }
    }

    let lhs = DualNumber::new(NonCommutative(1), NonCommutative(4));
    let rhs = DualNumber::new(NonCommutative(2), NonCommutative(3));

    assert_eq!(
        lhs * rhs,
        DualNumber::new(NonCommutative(4), NonCommutative(16))
    );
    assert_eq!(
        rhs * lhs,
        DualNumber::new(NonCommutative(3), NonCommutative(16))
    );
}

#[test]
fn scalar() {
    let lhs = DualNumber::new(1.0, 2.0);
    let scalar = 2.0;

    assert_eq!(scalar * lhs, lhs * scalar);

    assert_eq!(scalar * lhs, DualNumber::new(1.0 * scalar, 2.0 * scalar));
}
