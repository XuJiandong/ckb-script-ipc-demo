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
    pub fn execute<'de, Req, Resp, S>(self, _serve: S)
    where
        Req: Serialize + for<'d> Deserialize<'de>,
        Resp: Serialize + for<'d> Deserialize<'de>,
        S: Serve<'de, Req = Req, Resp = Resp>,
    {
        todo!()
    }
    pub fn send_request<Req: Serialize>(&mut self, req: Req) -> Result<(), IpcError> {
        let serialized_req = to_vec(&req, false).map_err(|_| IpcError::SerializeError)?;
        let packet = RequestPacket::new(serialized_req);
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
        let bytes = packet.serialize();
        self.writer.write(&bytes)?;
        Ok(())
    }
    pub fn receive_request<Req: for<'de> Deserialize<'de>>(&mut self) -> Result<Req, IpcError> {
        let packet = RequestPacket::read_from(&mut self.reader)?;
        let req = from_slice(&packet.payload(), false).map_err(|_| IpcError::DeserializeError)?;
        Ok(req)
    }
    pub fn receive_response<Resp: for<'de> Deserialize<'de>>(&mut self) -> Result<Resp, IpcError> {
        let packet = ResponsePacket::read_from(&mut self.reader)?;
        let resp = from_slice(&packet.payload(), false).map_err(|_| IpcError::DeserializeError)?;
        Ok(resp)
    }
}
