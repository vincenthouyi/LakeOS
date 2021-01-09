pub macro BIT($x:expr) {
    1 << $x
}

pub macro MASK($x:expr) {
    BIT!($x) - 1
}
