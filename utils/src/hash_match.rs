//! Match many keys from a `HashSet` (or any other method which has a `contains`
//! method) and perform an expression. Evaluates from top to bottom.
//!
//! # Example
//! ```rust
//! # use std::collections::HashSet;
//! # use utils::hash_matches;
//! # use utils::hash_match;
//!
//! let mut set = HashSet::new();
//! set.insert(0);
//!
//! let mut x = 0;
//! hash_match!(set: &0 => x = 1 );
//!
//! assert_eq!(x, 1);
//!
//! set.insert(1);
//! hash_matches!(set:
//!     &2 => x = 2,
//!     &0 => x = 4,
//!     &1 => x = 3,
//! );
//!
//! assert_eq!(x, 3);
//! ```

/// Match many keys from a `HashSet` (or any other method which has a `contains`
/// method) and perform an expression. Evaluates from top to bottom.
///
/// # Example
/// ```rust
/// # use std::collections::HashSet;
/// # use utils::hash_matches;
/// # use utils::hash_match;
///
/// let mut set = HashSet::new();
/// set.insert(0);
///
/// let mut x = 0;
/// hash_match!(set: &0 => x = 1 );
///
/// assert_eq!(x, 1);
///
/// set.insert(1);
/// hash_matches!(set:
///     &2 => x = 2,
///     &0 => x = 4,
///     &1 => x = 3,
/// );
///
/// assert_eq!(x, 3);
/// ```
#[macro_export]
macro_rules! hash_match {
    ($table:ident : $name:expr => $func:expr) => {
        if $table.contains($name) {
            $func
        }
    };
}

/// Match many keys from a `HashSet` (or any other method which has a `contains`
/// method) and perform an expression. Evaluates from top to bottom.
///
/// # Example
/// ```rust
/// # use std::collections::HashSet;
/// # use utils::hash_matches;
/// # use utils::hash_match;
///
/// let mut set = HashSet::new();
/// set.insert(0);
///
/// let mut x = 0;
/// hash_match!(set: &0 => x = 1 );
///
/// assert_eq!(x, 1);
///
/// set.insert(1);
/// hash_matches!(set:
///     &2 => x = 2,
///     &0 => x = 4,
///     &1 => x = 3,
/// );
///
/// assert_eq!(x, 3);
/// ```
#[macro_export]
macro_rules! hash_matches {
    ($table:ident : $($name:expr => $func:expr),+ $(,)?) => {
        $($crate::hash_match!($table: $name => $func); )+
    };
}

#[cfg(test)]
mod test {

    #[test]
    fn single() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(0);

        let mut x = 0;
        hash_match!(set: &0 => x = 1 );

        assert_eq!(x, 1);

        set.insert(1);
        hash_matches!(set:
            &2 => x = 2,
            &0 => x = 4,
            &1 => x = 3,
        );

        assert_eq!(x, 3);
    }
}
