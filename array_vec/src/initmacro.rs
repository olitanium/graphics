#[macro_export]
macro_rules! arrvec {
    ($($elem:expr),* $(,)?) => {
        ArrayVec::const_collect([$($elem,)*])
    };

    ($len:literal; $($elem: expr),* $(,)?) => {
        {
            let array: ArrayVec<_, $len> = arrvec![$($elem,)*];
            array
        }
    }
}