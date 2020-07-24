#![feature(decl_macro)]
#![feature(asm)]
#![feature(const_fn)]

#![no_std]

extern crate alloc;
extern crate naive;

mod console;
mod gpio;
mod timer;
mod rt;

use rustyl4api::object::{Capability, EndpointObj};

use naive::space_manager::gsm;

mod prelude {
    pub use crate::console::{print, println};
}

static RPI3B_ELF: &'static [u8] = include_bytes!("../build/shell.elf");

use prelude::*;

static mut EP: Option<Capability<EndpointObj>> = None;

// fn test_thread() -> ! {
//     for i in 1..=1 {
//         for _ in 0..10000000 {rustyl4api::syscall::nop()}
//         println!("妈妈再爱我{}次", i);
//     }

//     let mut buf = [0usize; 5];
//     let recved_buf = unsafe {
//         EP.as_ref().unwrap().receive(&mut buf)
//     }.unwrap();

//     println!("receive buf {:?}", recved_buf);
//     loop {}
// }

// fn spawn_test() {

//     naive::thread::spawn(test_thread);

//     let ep_cap = gsm!().alloc_object::<EndpointObj>(12)
//                            .unwrap();

//     unsafe {
//         EP = Some(ep_cap);
//     }

//     let buf = [10usize, 11];
//     println!("sending buf {:?}", buf);
//     unsafe {
//         EP.as_ref().unwrap().send(&buf).unwrap();
//     }
//     println!("after sending");
// }

// fn vm_test() {
//     use alloc::vec::Vec;

//     let mut vec = Vec::<usize>::new();

//     for i in 0..512 {
//         vec.push(i);
//     }

//     for (i, num) in vec.iter().enumerate() {
//         rustyl4api::kprintln!("vec[{}]: {}", i, num);
//     }
// }

// fn timer_test() {
//     for i in 0..5 {
//         println!("timer {}: {}", i, timer::current_time());
//         timer::spin_sleep_ms(1000);
//     }

//     // works now, but we don't have interrupt handling at the moment
// //    system_timer::tick_in(1000);
// }

use rustyl4api::object::{EpCap};

use rustyl4api::kprintln;
fn handle_console_request(ep: EpCap, incoming_badge: usize) -> ! {
    use rustyl4api::ipc::IpcMessage;
    use hashbrown::HashMap;

    let mut connections = HashMap::new();
    
    let listener = naive::urpc::UrpcListener::bind(ep.clone(), incoming_badge).unwrap();

    let mut recv_slot = gsm!().cspace_alloc().unwrap();
    let mut ret = ep.receive(Some(recv_slot));
    while let Ok(IpcMessage::Message{payload, need_reply, cap_transfer, badge}) = ret {
        if let Some(b) = badge {
            if b == incoming_badge {
                let c_ntf_cap = EpCap::new(recv_slot);
                let (mut stream, conn_badge) = listener.accept_with(c_ntf_cap).unwrap();
                stream.sleep_on_read();
                stream.sleep_on_write();
                connections.insert(conn_badge, stream);
                recv_slot = gsm!().cspace_alloc().unwrap();
                ret = ep.receive(Some(recv_slot));
            } else if let Some(stream) = connections.get_mut(&b) {
                let direction = payload[0];
                if direction == 0 {
                    let mut buf = [0; 100];
                    let readlen = stream.try_read_bytes(&mut buf).unwrap();
                    for byte in buf[..readlen].iter() {
                        console::CONSOLE.lock().write_byte(*byte);
                    }

                    ret = ep.receive(Some(recv_slot));
                } else if direction == 1 {
                    let mut buf = alloc::vec::Vec::new();
                    while let Some(byte) = console::CONSOLE.lock().try_read_byte() {
                        buf.push(byte);
                    }
                    if buf.len() > 0 {
                        stream.write_bytes(&buf);
                    }

                    ret = ep.receive(Some(recv_slot));
                }
            } else {
                kprintln!("warning: received badge not registered: {}", b);
                ret = ep.receive(Some(recv_slot));
            }
        } else {
            kprintln!("warning: receive unbadged message");
            ret = ep.receive(Some(recv_slot));
        }
    }

    loop {}
}

#[no_mangle]
pub fn main() {
    rustyl4api::kprintln!("Long may the sun shine!");

    gpio::init_gpio_server();

    console::init_console_server();

    timer::init_timer_server();

//    timer_test();

//    vm_test();

//    spawn_test();

    let incoming_badge = 1234;
    let ep = gsm!().alloc_object::<EndpointObj>(12).unwrap();
    let incoming_ep_slot = gsm!().cspace_alloc().unwrap();
    ep.mint(incoming_ep_slot, incoming_badge).unwrap();
    let incoming_ep = EpCap::new(incoming_ep_slot);

    naive::process::ProcessBuilder::new(&RPI3B_ELF)
        .stdin(incoming_ep.clone())
        .stdout(incoming_ep.clone())
        .stderr(incoming_ep.clone())
        .spawn()
        .expect("spawn process failed");

    handle_console_request(ep, 1234);
}