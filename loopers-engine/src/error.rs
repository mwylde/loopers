use std::io;

#[derive(Debug)]
pub enum SaveLoadError {
    HoundError(hound::Error),
    IOError(io::Error),
    OtherError(String),
    LooperSaveError(u32),
    LooperTimeoutError,
    ChannelFull,
    ChannelClosed,
}

impl From<hound::Error> for SaveLoadError {
    fn from(err: hound::Error) -> Self {
        SaveLoadError::HoundError(err)
    }
}

impl From<io::Error> for SaveLoadError {
    fn from(err: io::Error) -> Self {
        SaveLoadError::IOError(err)
    }
}
