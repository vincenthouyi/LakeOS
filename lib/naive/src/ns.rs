use alloc::sync::Arc;

use spin::Mutex;

use crate::ep_server::EP_SERVER;
use crate::rpc::RpcClient;

lazy_static! {
    static ref NS_CLIENT: Arc<Mutex<RpcClient>> = {
        let (ntf_badge, ntf_ep) = EP_SERVER.derive_badged_cap().unwrap();
        let inner = RpcClient::connect(
            &crate::space_manager::NAME_SERVICE_CAP,
            ntf_ep,
            ntf_badge,
        )
        .unwrap();
        Arc::new(Mutex::new(inner))
    };
}

pub fn ns_client() -> Arc<Mutex<RpcClient>> {
    NS_CLIENT.clone()
}
