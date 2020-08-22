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

use rustyl4api::object::{EndpointObj, InterruptCap};

use naive::space_manager::gsm;
use naive::ep_server::{EP_SERVER, EpServer};
use naive::urpc::{UrpcListener, UrpcListenerHandle};
use naive::urpc::stream::{UrpcStreamHandle};

use futures_util::StreamExt;
use spin::Mutex;

static SHELL_ELF: &'static [u8] = include_bytes!("../build/shell.elf");

// fn timer_test() {
//     for i in 0..5 {
//         println!("timer {}: {}", i, timer::current_time());
//         timer::spin_sleep_ms(1000);
//     }

//     // works now, but we don't have interrupt handling at the moment
// //    system_timer::tick_in(1000);
// }


static URPC_LISTENER: Mutex<Option<UrpcListenerHandle>> = Mutex::new(None);

async fn get_stream(listener: &UrpcListenerHandle) -> Vec<UrpcStreamHandle> {
    let mut ret = Vec::new();

    ret.push(listener.accept().await.unwrap());
    ret.push(listener.accept().await.unwrap());

    ret
}

async fn read_stream(streams: Vec<UrpcStreamHandle>) {
    use futures_util::stream::select_all;

    let mut merged = select_all(streams.into_iter());

    while let Some(b) = merged.next().await {
        console::console().poll_write(&[b]).await;
    }
}

async fn write_stream(streams: Vec<UrpcStreamHandle>) {

    let con = console::console();
    let mut con_stream = con.stream();

    while let Some(b) = con_stream.next().await {
        streams[1].poll_write(&[b]).await.unwrap();
    }
}

async fn worker_main() {
    use crate::futures_util::FutureExt;

    let streams = {
        let guard = URPC_LISTENER.lock();
        get_stream(guard.as_ref().unwrap()).await
    };

    let read_stream = read_stream(streams.clone()).fuse();
    let write_stream = write_stream(streams.clone()).fuse(); 

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

    let ep = gsm!().alloc_object::<EndpointObj>(12).unwrap();

    let ep_server = EP_SERVER.try_get_or_init(|| EpServer::new(ep)).unwrap();
    let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();

    naive::process::ProcessBuilder::new(&SHELL_ELF)
        .stdin(listen_ep.clone())
        .stdout(listen_ep.clone())
        .stderr(listen_ep.clone())
        .spawn()
        .expect("spawn process failed");

    let con = console::console_server();
    let (irq_badge, irq_ep) = ep_server.derive_badged_cap().unwrap();
    let irq_cntl_cap = InterruptCap::new(rustyl4api::init::InitCSpaceSlot::IrqController as usize);
    irq_cntl_cap.attach_ep_to_irq(irq_ep.slot, pi::interrupt::Interrupt::Aux as usize).unwrap();
    ep_server.insert_notification(pi::interrupt::Interrupt::Aux as usize, Box::new(con));

    let listener = UrpcListener::bind(listen_ep, listen_badge).unwrap();
    let listener = UrpcListenerHandle::from_listener(listener);
    *URPC_LISTENER.lock() = Some(listener.clone());
    // let listener = Box::new(UrpcConnectionHandler{inner:   });
    ep_server.insert_event(listen_badge, Box::new(listener));

    naive::thread::spawn(worker_thread);

    ep_server.run();

    loop {}
}