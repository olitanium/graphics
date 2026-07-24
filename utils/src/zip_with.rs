pub fn zip_map_arrays<T, U, V, F: FnMut(T, U) -> V, const N: usize>(
    array1: [T; N],
    array2: [U; N],
    mut f: F,
) -> [V; N] {
    let mut zipped = std::iter::zip(array1, array2);

    core::array::from_fn(|_| {
        let (a, b) = unsafe { zipped.next().unwrap_unchecked() };
        f(a, b)
    })
}

pub trait ArrayZipWith<const N: usize, T> {
    fn zip_with<U, V, F: FnMut(T, U) -> V>(self, array2: [U; N], f: F) -> [V; N];
}

impl<const N: usize, T> ArrayZipWith<N, T> for [T; N] {
    fn zip_with<U, V, F: FnMut(T, U) -> V>(self, array2: [U; N], f: F) -> [V; N] {
        zip_map_arrays(self, array2, f)
    }
}
