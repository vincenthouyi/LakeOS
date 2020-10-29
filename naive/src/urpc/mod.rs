
pub mod listener;
pub mod stream;

pub use listener::{UrpcListener, UrpcListenerHandle};
pub use stream::{UrpcStream, UrpcStreamChannel, UrpcStreamHandle, Role};

pub trait UrpcHandler {
    fn new(channel: UrpcStreamChannel) -> Self;
}