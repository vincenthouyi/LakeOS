use conquer_once::spin::OnceCell;
use spin::Mutex;

use futures_util::AsyncReadExt;

use crate::fs::File;

static TIME_CLIENT: OnceCell<Mutex<File>> = OnceCell::uninit();

async fn time_client() -> &'static Mutex<File> {
    let time_client = if TIME_CLIENT.get().is_none() {
        let timer_fd = File::open(&"/dev/timer").await.unwrap();
        Some(Mutex::new(timer_fd))
    } else {
        None
    };

    TIME_CLIENT.get_or_init(|| time_client.unwrap())
}

pub async fn current_time() -> u64 {
    let mut time_buf = [0; 8];
    time_client().await.lock().read(&mut time_buf).await.unwrap();
    unsafe {
        core::mem::transmute(time_buf)
    }
}

pub async fn sleep_us(us: u64) {
    // QEMU for rpi3 do not support system timer irq. spin waiting for now.
    let cur = current_time().await;

    while current_time().await < cur + us {}
}

pub async fn sleep_ms(ms: u64) {
    sleep_us(ms * 1000).await
}
