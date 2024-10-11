use ckb_std::error::SysError;

#[repr(i8)]
pub enum Error {
    Unknown = 1,
    CkbSysError(SysError),
    ServerError,
}
