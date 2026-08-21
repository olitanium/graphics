use utils::error_boilerplate;

#[derive(Debug)]
pub enum TryFromError<T> {
    High(T),
    Low(T),
}

error_boilerplate!({T} TryFromError<T>);
