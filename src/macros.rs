#![macro_use]

macro_rules! err {
    ($expr:expr) => (
        return Err(::std::convert::From::from($expr));
    )
}
