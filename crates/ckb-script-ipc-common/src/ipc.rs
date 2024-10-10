use crate::error::IpcError;
use serde::{Deserialize, Serialize};

pub trait Serve<'a> {
    /// Type of request.
    type Req: Serialize;

    /// Type of response.
    type Resp: Deserialize<'a>;

    /// Responds to a single request.
    fn serve(self, req: Self::Req) -> Result<Self::Resp, IpcError>;

    /// Extracts a method name from the request.
    fn method(&self, _request: &Self::Req) -> Option<&'static str> {
        None
    }
}
