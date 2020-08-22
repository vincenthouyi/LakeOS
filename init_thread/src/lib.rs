#![feature(decl_macro)]
#![feature(asm)]
#![feature(const_fn)]

#![no_std]

extern crate alloc;
extern crate naive;
#[macro_use] extern crate futures_util;
#[macro_use] extern crate rustyl4api;

mod console;
mod gpio;
// mod timer;
mod rt;

use alloc::boxed::Box;
use alloc::vec::Vec;

use rustyl4api::object::{EndpointObj};
use rustyl4api::ipc::IpcMessage;

use naive::space_manager::gsm;
use naive::ep_server::{EpServer, EpMsgHandler};
use naive::urpc::{UrpcListener};
use naive::urpc::stream::UrpcStreamExt;

use futures_util::StreamExt;

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
use spin::Mutex;

struct UrpcConnectionHandler {
    inner: UrpcListener
}

impl EpMsgHandler for UrpcConnectionHandler {
    fn handle_ipc(&self, ep_server: &EpServer, msg: IpcMessage, cap_transfer_slot: Option<usize>) {
        let c_ntf_cap = EpCap::new(cap_transfer_slot.unwrap());
        let (conn_badge, s_ntf_cap) = ep_server.derive_badged_cap().unwrap();
        let stream_inner = self.inner.accept_with(c_ntf_cap, s_ntf_cap).unwrap();
        let stream = UrpcStreamExt::from_stream(stream_inner);
        STREAM.lock().push(stream.clone());
        ep_server.insert_event(conn_badge, Box::new(stream));
    }
}

static STREAM: Mutex<Vec<UrpcStreamExt>> = Mutex::new(Vec::new());

fn get_stream() -> Vec<UrpcStreamExt> {
    loop {
        let streams = STREAM.lock();
        if streams.len() != 2 {
            continue;
        }

        let mut v = Vec::new();
        for s in streams.iter() {
            v.push(s.clone());
        }
        return v;
    }
}

async fn read_stream() {
    use futures_util::stream::select_all;

    let readers = get_stream().into_iter().map(|x| x.reader());
    let mut merged = select_all(readers);

    while let Some(b) = merged.next().await {
        console::console().poll_write(&[b]).await;
    }
}

async fn write_stream() {

    let con = console::console();
    let mut con_stream = con.stream();
    let streams = get_stream();

    while let Some(b) = con_stream.next().await {
        streams[1].poll_write(&[b]).await.unwrap();
    }
}

async fn worker_main() {
    use crate::futures_util::FutureExt;

    let read_stream = read_stream().fuse();
    let write_stream = write_stream().fuse(); 

    pin_mut!(read_stream, write_stream);

    loop {
        select! {
            () = read_stream => { },
            () = write_stream => { },
            complete => { break }
        }
    }
}

pub fn worker_thread() -> ! {
    use naive::task::Task;

    let mut exe = naive::task::Executor::new();
<<<<<<< HEAD
    exe.spawn(Task::new(console::read_from_uart()));
    exe.spawn(Task::new(console::write_to_uart()));
=======
    exe.spawn(Task::new(worker_main()));
>>>>>>> 43dc10a... use modified futures-util async console io
    exe.run();

    loop {}
}

#[no_mangle]
pub fn main() {
    kprintln!("Long may the sun shine!");

    gpio::init_gpio_server();

    // timer::init_timer_server();

//    timer_test();

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

    let con = console::console_server();
    let (irq_badge, irq_ep) = ep_server.derive_badged_cap().unwrap();
    use rustyl4api::object::{Capability, InterruptObj};
    let irq_cntl_cap = Capability::<InterruptObj>::new(rustyl4api::init::InitCSpaceSlot::IrqController as usize);
    irq_cntl_cap.attach_ep_to_irq(irq_ep.slot, pi::interrupt::Interrupt::Aux as usize).unwrap();
    ep_server.insert_notification(pi::interrupt::Interrupt::Aux as usize, Box::new(con));

    let listener = Box::new(UrpcConnectionHandler{inner:  UrpcListener::bind(listen_ep, listen_badge).unwrap() });
    ep_server.insert_event(listen_badge, listener);

    ep_server.run();

    loop {}
}