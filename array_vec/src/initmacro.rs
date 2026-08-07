/// Macro for creating an `ArrayVec` without needing to push each element.
#[macro_export]
macro_rules! arrvec {
    ($($elem:expr),* $(,)?) => {
        $crate::ArrayVec::const_collect([$($elem,)*])
    };

    ($($elem: expr),* $(,)? ; $len:literal ) => {
        {
            let array: $crate::ArrayVec<_, $len> = arrvec![$($elem,)*];
            array
        }
    }
}
