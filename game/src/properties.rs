use engine::SimpleEntity;

pub trait Properties {
    fn is_hookpoint(&self) -> bool {
        false
    }

    fn is_carryable(&self) -> bool {
        false
    }
}

impl<T> Properties for SimpleEntity<T> {}
