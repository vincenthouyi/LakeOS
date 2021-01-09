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

#[macro_export]
macro_rules! SYMBOL {
    ($x:expr) => {
        unsafe { ($x).as_ptr() as usize }
    };
}

#[macro_export]
macro_rules! ALIGNUP {
    ($addr:expr, $sz: expr) => {
        (($addr) + BIT!($sz) - 1) & (!MASK!($sz))
    };
}

#[macro_export]
macro_rules! current {
    () => {
        SCHEDULER.current().unwrap()
    };
}
