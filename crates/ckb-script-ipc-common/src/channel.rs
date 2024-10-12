use crate::io::Write;
use crate::ipc::Serve;
use crate::packet::{Packet, RequestPacket, ResponsePacket};
use crate::{error::IpcError, pipe::Pipe};
use serde::{Deserialize, Serialize};
use serde_molecule::{from_slice, to_vec};

pub struct Channel {
    reader: Pipe,
    writer: Pipe,
}

impl Channel {
    pub fn new(reader: Pipe, writer: Pipe) -> Self {
        Self { reader, writer }
    }
}

impl Channel {
    /// Execute a server loop
    /// 1. receive request
    /// 2. call serve method
    /// 3. send response
    /// 4. continue
    pub fn execute<Req, Resp, S>(mut self, serve: &mut S) -> Result<(), IpcError>
    where
        Req: Serialize + for<'de> Deserialize<'de>,
        Resp: Serialize + for<'de> Deserialize<'de>,
        S: Serve<Req = Req, Resp = Resp>,
    {
        loop {
            let req: Req = self.receive_request()?;
            let resp = serve.serve(req)?;
            self.send_response(0, resp)?;
        }
    }
    // used for client
    pub fn call<Req, Resp>(
        &mut self,
        _method_name: &'static str,
        req: Req,
    ) -> Result<Resp, IpcError>
    where
        Req: Serialize + for<'de> Deserialize<'de>,
        Resp: Serialize + for<'de> Deserialize<'de>,
    {
        // TODO: method name can be used for debugging
        self.send_request(req)?;
        self.receive_response()
    }
    pub fn send_request<Req: Serialize>(&mut self, req: Req) -> Result<(), IpcError> {
        let serialized_req = to_vec(&req, false).map_err(|_| IpcError::SerializeError)?;
        let packet = RequestPacket::new(serialized_req);
        #[cfg(feature = "enable-logging")]
        log::info!("send request: {:?}", packet);

        let bytes = packet.serialize();
        self.writer.write(&bytes)?;
        Ok(())
    }
    pub fn send_response<Resp: Serialize>(
        &mut self,
        error_code: u64,
        resp: Resp,
    ) -> Result<(), IpcError> {
        let serialized_resp = to_vec(&resp, false).map_err(|_| IpcError::SerializeError)?;
        let packet = ResponsePacket::new(error_code, serialized_resp);
        #[cfg(feature = "enable-logging")]
        log::info!("send response: {:?}", packet);

        let bytes = packet.serialize();
        self.writer.write(&bytes)?;
        Ok(())
    }
    pub fn receive_request<Req: for<'de> Deserialize<'de>>(&mut self) -> Result<Req, IpcError> {
        let packet = RequestPacket::read_from(&mut self.reader)?;
        #[cfg(feature = "enable-logging")]
        log::info!("receive request: {:?}", packet);

        let req = from_slice(packet.payload(), false).map_err(|_| IpcError::DeserializeError)?;
        Ok(req)
    }
    pub fn receive_response<Resp: for<'de> Deserialize<'de>>(&mut self) -> Result<Resp, IpcError> {
        let packet = ResponsePacket::read_from(&mut self.reader)?;
        #[cfg(feature = "enable-logging")]
        log::info!("receive response: {:?}", packet);

        let resp = from_slice(packet.payload(), false).map_err(|_| IpcError::DeserializeError)?;
        Ok(resp)
    }
}
