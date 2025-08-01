#[test]
fn pop_push() {
    use super::ArrayVec;

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
    use super::ArrayVec;

    let mut vec: ArrayVec<_, 10> = (0..20_usize).collect();
    
    assert_eq!(vec.pop(), Some(9));

    for (index, item) in vec.into_iter().enumerate() {
        assert_eq!(item, index);
    }
}

#[test]
fn rev() {
    use super::ArrayVec;

    let vec: ArrayVec<_, 10> = (0..10_usize).collect();

    for (index, item) in vec.into_iter().rev().enumerate() {
        assert_eq!(9 - item, index);
    }


}


