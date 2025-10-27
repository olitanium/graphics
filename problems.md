playground: https://play.rust-lang.org/?version=nightly&mode=debug&edition=2024&gist=9e5f6e37b3b79d85881b71dfce9ec967

I have a crate-level error type which is exposed to the library user, which features a generic variant to store a user's own personal error type. This would ideally allow for interoperability where the ? operator will report errors upwards to the 

```rust
#![feature(never_type)]

pub enum MyError<T> {
    Variant,
    //...
    User(T),
}

pub fn internal_or_external() -> Result<(), MyError<!>> {
    Ok(())
}

pub fn from_user<T>(f: fn() -> Result<(), MyError<T>>) -> Result<(), MyError<T>> {
    //internal_or_external()?; // the trait `From<MyError<!>>` is not implemented for `MyError<T>`
    f()
}
```

The obvious solution does not work because of the overlap `impl<T> for T {/*...*/}`

```rust
impl<T /*not `!`*/> From<MyError<!>> for MyError<T> {
    fn from(value: MyError<!>) -> Self {
        match value {
            MyError::Variant => Self::Variant
            //... => ...
        }
    }
}
```
## Imperfect Solution 1

Use generics for every internal function, which misses the contextual impact of the `never` type

This may cause monomorphisation overhead when a function is called from multiple contexts, even though each invocation does no different work

```rust
fn internal_generic<T>() -> Result<(), MyError<T>> {
    Ok(())
}

pub fn from_user_generics<T>(f: fn() -> Result<(), MyError<T>>) -> Result<(), MyError<T>> {
    internal_generic()?;
    f()
}
```


## Imperfect Solution 2

Use a conversion function not reliant on `Into`, and `map_err` each occasion. This would have to be exposed to the end user in case they decide to use the `internal_or_external` functions too

```rust
fn temp_convert<T>(value: MyError<!>) -> MyError<T> {
    match value {
        MyError::Variant => MyError::Variant
        //... => ...
    }
}

pub fn from_user_with_convert<T>(f: fn() -> Result<(), MyError<T>>) -> Result<(), MyError<T>> {
    internal_or_external().map_err(temp_convert)?;
    f()
}
```

## Question

Does anyone have a solution which is as slick as possible for the end users who need to call these `internal_or_external` style functions.