#![feature(decl_macro)]
#![feature(asm)]
#![feature(const_fn)]

#![no_std]

extern crate alloc;
extern crate naive;
#[macro_use] extern crate rustyl4api;

mod console;
mod gpio;
mod timer;
mod rt;

use alloc::boxed::Box;

use rustyl4api::object::{EndpointObj};
use rustyl4api::ipc::IpcMessage;

use naive::space_manager::gsm;
use naive::ep_server::{EpServer, Context};
use naive::urpc::{UrpcListener, UrpcStream};

static SHELL_ELF: &'static [u8] = include_bytes!("../build/shell.elf");

// fn timer_test() {
//     for i in 0..5 {
//         println!("timer {}: {}", i, timer::current_time());
//         timer::spin_sleep_ms(1000);
//     }

//     // works now, but we don't have interrupt handling at the moment
// //    system_timer::tick_in(1000);
// }

use rustyl4api::object::{EpCap};

fn urpc_notification_handler(ep_server: &EpServer, msg: IpcMessage, cap_transfer_slot: Option<usize>, ctx: Context) {
    if let IpcMessage::Message{payload, need_reply, cap_transfer, badge} = msg {
        let direction = payload[0];
        let mut stream = {
            if let Context::Pointer(ptr) = ctx {
                unsafe{ Box::from_raw(ptr as *mut UrpcStream) }
            } else {
                panic!()
            }
        };
        if direction == 0 {
            let mut buf = [0; 100];
            let readlen = stream.try_read_bytes(&mut buf).unwrap();
            for byte in buf[..readlen].iter() {
                console::tx_buf().push(*byte);
            }
        } else if direction == 1 {
            let mut buf = alloc::vec::Vec::new();
            while let Ok(byte) = console::rx_buf().pop() {
                buf.push(byte);
            }
            if buf.len() > 0 {
                stream.write_bytes(&buf).unwrap();
            }
        }

        core::mem::forget(stream);
    }
}

fn connection_handler(ep_server: &EpServer, msg: IpcMessage, cap_transfer_slot: Option<usize>, ctx: Context) {
    //TODO: param checking
    let listener = {
        if let Context::Pointer(ptr) = ctx {
            unsafe{ Box::from_raw(ptr as *mut UrpcListener) }
        } else {
            panic!()
        }
    };
    let c_ntf_cap = EpCap::new(cap_transfer_slot.unwrap());
    let (conn_badge, s_ntf_cap) = ep_server.derive_badged_cap().unwrap();
    let mut stream = Box::new(listener.accept_with(c_ntf_cap, s_ntf_cap).unwrap());
    stream.sleep_on_read();
    stream.sleep_on_write();
    ep_server.insert_event(conn_badge, urpc_notification_handler, Context::Pointer(Box::into_raw(stream) as usize));

    core::mem::forget(listener);
}

pub fn worker_thread() -> ! {
    use naive::task::{Executor, Task};

    let mut exe = Executor::new();
    exe.spawn(Task::new(console::read_from_uart()));
    exe.spawn(Task::new(console::write_to_uart()));
    exe.run();

    loop {}
}

#[no_mangle]
pub fn main() {
    kprintln!("Long may the sun shine!");

    gpio::init_gpio_server();

    // console::init_console_server();

    timer::init_timer_server();

//    timer_test();

    console::console();

    naive::thread::spawn(worker_thread);

    let ep = gsm!().alloc_object::<EndpointObj>(12).unwrap();

    let ep_server = EpServer::new(ep);
    let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();

    naive::process::ProcessBuilder::new(&SHELL_ELF)
        .stdin(listen_ep.clone())
        .stdout(listen_ep.clone())
        .stderr(listen_ep.clone())
        .spawn()
        .expect("spawn process failed");

    let listener = Box::new(UrpcListener::bind(listen_ep, listen_badge).unwrap());
    ep_server.insert_event(listen_badge, connection_handler, Context::Pointer(Box::into_raw(listener) as usize));

    ep_server.run();

    loop {}
}