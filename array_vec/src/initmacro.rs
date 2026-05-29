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
