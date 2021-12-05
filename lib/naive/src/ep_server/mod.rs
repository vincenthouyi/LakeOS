
mod ep_server;
mod msg_receiver;
// mod fault_receiver;

pub use ep_server::{EpServer, MessageHandler, NotificationHandler};
pub use msg_receiver::MsgReceiver;
// pub use fault_receiver::FaultReceiver;

lazy_static! {
    pub static ref EP_SERVER: EpServer = {
        use crate::space_manager::gsm;
        use crate::objects::EndpointObj;

        let ep = gsm!().alloc_object::<EndpointObj>(12).unwrap();
        EpServer::new(ep)
    };
}