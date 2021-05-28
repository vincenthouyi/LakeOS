#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Error {
    NotSupported,
    Invalid,
    InternalError,
    ProtocolError,
    NoReceiver,
    NoMemory,
}

pub type Result<T> = core::result::Result<T, Error>;
