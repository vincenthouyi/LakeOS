#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::_print(format_args!($($arg)*)))
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => (async {
        $crate::io::_print(format_args!($($arg)*)).await;
        $crate::io::_print(format_args!("\n")).await;
    })
}
