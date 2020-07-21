
#[no_mangle]
fn main() -> () {
    println!("hello world");

    for i in 0..10 {
        let a = std::boxed::Box::new(i);
        println!("i {}", a);
    }

    eprintln!("here is eprint");
    loop {};
}