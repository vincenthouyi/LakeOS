use alloc::sync::Arc;

use spin::Mutex;

use crate::ep_server::EP_SERVER;
use crate::rpc::RpcClient;

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    Success,
    ServiceNotFound,
}

impl Error {
    pub fn into_result(self) -> Result<()> {
        match self {
            Error::Success => Ok(()),
            e => Err(e),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

lazy_static! {
    static ref NS_CLIENT: Arc<Mutex<RpcClient>> = {
        let ep_server = EP_SERVER.try_get().unwrap();
        let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
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
