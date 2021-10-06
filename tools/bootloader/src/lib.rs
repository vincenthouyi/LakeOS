#![no_std]

#[macro_use]
mod console {
    #[macro_export]
    macro_rules! kprint {
        () => {};
        ($($arg:tt)*) => {};
    }

    #[macro_export]
    macro_rules! kprintln {
        () => {};
        ($($arg:tt)*) => {};
    }
}

pub mod boot_info;
