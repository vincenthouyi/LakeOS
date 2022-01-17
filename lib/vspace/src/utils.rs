#[macro_export]
macro_rules! BIT {
    ($x:expr) => {
        1 << $x
    };
}

#[macro_export]
macro_rules! MASK {
    ($x:expr) => {
        BIT!($x) - 1
    };
}
