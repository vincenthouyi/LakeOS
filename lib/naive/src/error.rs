#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Error {
    NotSupported,
    Invalid,
    InternalError,
}

pub type Result<T> = core::result::Result<T, Error>;
