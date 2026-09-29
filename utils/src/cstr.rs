trait Something {}

impl Something for core::ffi::CStr {}
impl Something for &core::ffi::CStr {}

fn test() {
    let x = &c"test" as &dyn Something;
}
