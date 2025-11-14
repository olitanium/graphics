use crate::{ArrayVec, arrvec};

#[test]
fn pop_push() {
    use crate::ArrayVec;

    let mut vec: ArrayVec<_, 10> = ArrayVec::new();
    vec.push(10_u32);
    assert_eq!(vec.pop(), Some(10_u32));
    for elem in 0..10 {
        vec.push(elem);
    }
    assert_eq!(vec.push_within_capacity(11), Err(11));

    for elem in (0..10).rev() {
        assert_eq!(vec.pop(), Some(elem));
    }

    assert_eq!(vec.pop(), None);
}

#[test]
fn iter() {
    use crate::ArrayVec;

    let mut vec: ArrayVec<_, 10> = (0..20_usize).collect();

    assert_eq!(vec.pop(), Some(9));

    for (index, item) in vec.into_iter().enumerate() {
        assert_eq!(item, index);
    }
}

#[test]
fn rev() {
    use crate::ArrayVec;

    let vec: ArrayVec<_, 10> = (0..10_usize).collect();

    for (index, item) in vec.into_iter().rev().enumerate() {
        assert_eq!(9 - item, index);
    }
}

#[test]
fn size() {
    // len = 3
    use crate::ArrayVec;
    let vec: ArrayVec<_, 10> = (0..3).collect();

    let mut iter = vec.into_iter();

    assert_eq!(iter.size_hint(), (3, Some(3)));

    iter.next();

    assert_eq!(iter.size_hint(), (2, Some(2)));
}

#[test]
fn init_macro() {
    let array: ArrayVec<u32, 4> = arrvec![1, 2];
    assert_eq!(array.as_slice(), &[1, 2]);

    //compile fail
    //const ARRAY: ArrayVec<u32, 1> = arrvec![1, 2];
    
    let array: ArrayVec<u32, _> = arrvec![5; 1,2,3,];
    assert_eq!(array.capacity(), 5);
}