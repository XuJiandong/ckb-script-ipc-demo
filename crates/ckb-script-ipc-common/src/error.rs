use ckb_std::error::SysError;
use core::fmt::{self, Debug, Display};

// use core::error::Error when Rust 1.81 is used.
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub enum IpcError {
    CkbSysError(SysError),
    UnexpectedEof,
    IncompleteVlqSeq,
    DecodeVlqOverflow,
    ReadVlqError,
    SerializeError,
    DeserializeError,
    SliceWriteError,
    ReadUntilError,
    ReadExactError,
    BufReaderError,
}

impl Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for IpcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[repr(u64)]
pub enum ProtocolErrorCode {
    DeserializeError = 1,
    OtherEndClosed = 2,
}
