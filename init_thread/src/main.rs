#![feature(decl_macro)]
#![feature(asm)]
#![feature(const_fn)]

#![no_std]
#![no_main]

extern crate alloc;
extern crate naive;
#[macro_use] extern crate futures_util;
#[macro_use] extern crate rustyl4api;

mod console;
mod gpio;
// mod timer;
// mod rt;

use alloc::boxed::Box;
use alloc::vec::Vec;

use rustyl4api::object::{InterruptCap};

use naive::urpc::{UrpcListener, UrpcListenerHandle};
use naive::urpc::stream::{UrpcStreamHandle};
use naive::io::AsyncWriteExt;

use futures_util::StreamExt;

static SHELL_ELF: &'static [u8] = include_bytes!("../build/shell");

// fn timer_test() {
//     for i in 0..5 {
//         println!("timer {}: {}", i, timer::current_time());
//         timer::spin_sleep_ms(1000);
//     }

//     // works now, but we don't have interrupt handling at the moment
// //    system_timer::tick_in(1000);
// }

async fn get_stream(listener: &UrpcListenerHandle) -> Vec<UrpcStreamHandle> {
    let mut ret = Vec::new();

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

async fn write_stream(mut streams: Vec<UrpcStreamHandle>) {
    let con = console::console();
    let mut con_stream = con.stream();

    while let Some(b) = con_stream.next().await {
        streams[0].write(&[b]).await.unwrap();
    }
}

#[naive::main]
async fn main() {
    use crate::futures_util::FutureExt;

    gpio::init_gpio_server();

    let ep_server = EP_SERVER.try_get().unwrap();
    let (listen_badge, listen_ep) = ep_server.derive_badged_cap().unwrap();

    naive::process::ProcessBuilder::new(&SHELL_ELF)
        .stdio(listen_ep.clone())
        .spawn()
        .expect("spawn process failed");

    let con = console::console_server();
    let (irq_badge, irq_ep) = ep_server.derive_badged_cap().unwrap();
    let irq_cntl_cap = InterruptCap::new(rustyl4api::init::InitCSpaceSlot::IrqController as usize);
    irq_cntl_cap.attach_ep_to_irq(irq_ep.slot, pi::interrupt::Interrupt::Aux as usize).unwrap();
    ep_server.insert_notification(pi::interrupt::Interrupt::Aux as usize, Box::new(con));

    let listener = UrpcListener::bind(listen_ep, listen_badge).unwrap();
    let listener = UrpcListenerHandle::from_listener(listener);
    ep_server.insert_event(listen_badge, Box::new(listener.clone()));

    let streams = get_stream(&listener).await;

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