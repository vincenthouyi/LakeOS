use std::io::Read;

extern crate cpio;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mut file = std::fs::File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let cpio_file = cpio::NewcReader::from_bytes(&buf);
    for entry in cpio_file.entries() {
        println!("header size {} file sz {} tot sz {} {:?}", entry.header_size(), entry.file_size(), entry.total_size(), core::str::from_utf8(&entry.content()[0..10]).unwrap());
    }
}