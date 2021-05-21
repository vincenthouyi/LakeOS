use alloc::sync::Arc;

use spin::Mutex;

use crate::ep_server::EP_SERVER;
use crate::rpc::RpcClient;

// lazy_static! {
//     static ref NS_CLIENT: Arc<Mutex<RpcClient>> = {
//         let (ntf_badge, ntf_ep) = EP_SERVER.derive_badged_cap().unwrap();
//         let receiver = EpReceiver::new(ntf_ep.into(), ntf_badge);
//         let inner = RpcClient::connect(
//             &crate::space_manager::NAME_SERVICE_CAP,
//             receiver
//         )
//         .unwrap();
//         Arc::new(Mutex::new(inner))
//     };
// }

pub async fn ns_client() -> Arc<Mutex<RpcClient>> {
    // TODO: create one ns client every time this function is called.
    // Should find some way to lazily store the client in async context.
    let receiver = EP_SERVER.derive_receiver();
    let inner = RpcClient::connect(
        &crate::space_manager::NAME_SERVICE_CAP,
        receiver
    )
    .await
    .unwrap();
    Arc::new(Mutex::new(inner))
}
