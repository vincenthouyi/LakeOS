
struct Console {}

impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let stdout = rustyl4api::object::EpCap::new(rustyl4api::process::ProcessCSpace::Stdout as usize);

        for c in s.chars() {
            let buf = [c as usize];
            // rustyl4api::kprintln!("sending {:?}", buf);
            stdout.send(&buf).unwrap();
            // if c == '\n' {
            //     stdout.send(&['\r'.to_digit()]).unwrap()
            // }
            // stdout.send(&[c.into()]).unwrap()
        }
        Ok(())
    }
}

pub fn console_print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    let mut console = Console{};
    console.write_fmt(args);

    // stdout.send();
    // CONSOLE.lock().write_fmt(args).unwrap();
}


pub macro println {
    () => (print!("\n")),
    ($fmt:expr) => (print!(concat!($fmt, "\n"))),
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*))
}

pub macro print($($arg:tt)*) {
    console_print(format_args!($($arg)*))
}