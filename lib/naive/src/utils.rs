
/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub const fn align_down(addr: usize, align: usize) -> usize {
//    if !align.is_power_of_two() {
//        panic!("align is not power of 2");
//    }

    addr & !(align - 1)
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub const fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr.saturating_add(align - 1), align)
}

pub fn prev_power_of_two(num: usize) -> usize {
    use core::mem::size_of;

    1 << (8 * (size_of::<usize>()) - num.leading_zeros() as usize - 1)
}

#[macro_export]
macro_rules! BIT {
    ($x:expr) => (1 << $x);
}


#[macro_export]
macro_rules! MASK {
    ($x:expr) => (BIT!($x) - 1);
}