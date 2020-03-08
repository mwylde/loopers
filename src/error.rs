use std::io;

#[derive(Debug)]
pub enum SaveLoadError {
    HoundError(hound::Error),
    IOError(io::Error),
    OtherError(String),
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

impl From<prost::EncodeError> for SaveLoadError {
    fn from(err: prost::EncodeError) -> Self {
        SaveLoadError::OtherError(format!("Protobuf encoding error: {:?}", err))
    }
}

impl From<prost::DecodeError> for SaveLoadError {
    fn from(err: prost::DecodeError) -> Self {
        SaveLoadError::OtherError(format!("Protobuf decoding error: {:?}", err))
    }
}
