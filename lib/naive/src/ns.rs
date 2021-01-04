use alloc::sync::Arc;

use spin::Mutex;
use conquer_once::spin::OnceCell;

use crate::rpc::RpcClient;
use crate::ep_server::EP_SERVER;

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    Success,
    ServiceNotFound,
}

impl Error {
    pub fn into_result(self) -> Result<()> {
        match self {
            Error::Success => Ok(()),
            e => Err(e)
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub fn ns_client() -> Arc<Mutex<RpcClient>> {
    use rustyl4api::{process::ProcessCSpace, object::EpCap};
    static NS_CLIENT: OnceCell<Arc<Mutex<RpcClient>>> = OnceCell::uninit();

    NS_CLIENT.try_get_or_init(|| {
        let ep_server = EP_SERVER.try_get().unwrap();
        let (ntf_badge, ntf_ep) = ep_server.derive_badged_cap().unwrap();
        let inner = RpcClient::connect(EpCap::new(ProcessCSpace::NameServer as usize), ntf_ep, ntf_badge).unwrap();
        Arc::new(Mutex::new(inner))
    }).unwrap().clone()
}