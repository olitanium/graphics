#[macro_export]
macro_rules! arrvec {
    ($($elem:expr),* $(,)?) => {
        ArrayVec::const_collect([$($elem,)*])
    };

    ($($elem: expr),* $(,)? ; $len:literal ) => {
        {
            let array: ArrayVec<_, $len> = arrvec![$($elem,)*];
            array
        }
    }
}
