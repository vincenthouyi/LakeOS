use conquer_once::spin::OnceCell;
use spin::Mutex;

use rustyl4api::object::EpCap;

use crate::ep_server::EP_SERVER;
use crate::ns;
use crate::rpc::RpcClient;

static TIME_CLIENT: OnceCell<Mutex<RpcClient>> = OnceCell::uninit();

async fn time_client() -> &'static Mutex<RpcClient> {
    use crate::alloc::string::ToString;

    let time_client = if TIME_CLIENT.get().is_none() {
        let timer_cap_slot = {
            let mut slot: Option<usize> = None;
            while let None = slot {
                slot = ns::ns_client()
                    .lock()
                    .lookup_service("timer".to_string())
                    .await
                    .ok();
            }
            slot.unwrap()
        };
        let timer_cap = EpCap::new(timer_cap_slot);

        let ep_server = EP_SERVER.try_get().unwrap();
        let (cli_badge, cli_ep) = ep_server.derive_badged_cap().unwrap();

        Some(Mutex::new(
            RpcClient::connect(timer_cap, cli_ep, cli_badge).unwrap(),
        ))
    } else {
        None
    };

    TIME_CLIENT.get_or_init(|| time_client.unwrap())
}

pub async fn current_time() -> u64 {
    time_client().await.lock().current_time().await.unwrap()
}

pub async fn sleep_us(us: u64) {
    // QEMU for rpi3 do not support system timer irq. spin waiting for now.
    let cur = current_time().await;

    while current_time().await < cur + us {}
}

pub async fn sleep_ms(ms: u64) {
    sleep_us(ms * 1000).await
}
