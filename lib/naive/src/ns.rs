
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