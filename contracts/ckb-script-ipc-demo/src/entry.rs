use alloc::{ffi::CString, format, string::String};
use ckb_std::{ckb_constants::Source, env::argv, high_level::inherited_fds, log::info, logger};

use crate::error::Error;

// before proc-macro expansion
// #[derive(CkbScriptIpc)]
// trait World {
//     // note self is not used
//     fn hello(name: String) -> Result<(), String>;
// }
// will be expanded to the following:

// ---------------------------------
// start of auto generated code
// ---------------------------------
use ckb_script_ipc_common::{
    channel::Channel, error::IpcError, ipc::Serve, pipe::Pipe, spawn::spawn_server,
};
use serde::{Deserialize, Serialize};

trait World: Sized {
    fn hello(&self, name: String) -> Result<String, u64>;

    // added
    fn server(self) -> ServeWorld<Self> {
        ServeWorld { service: self }
    }
}

struct ServeWorld<S> {
    service: S,
}

impl<S> Serve for ServeWorld<S>
where
    S: World + Sized,
{
    type Req = WorldRequest;
    type Resp = WorldResponse;
    fn serve(&mut self, req: Self::Req) -> Result<Self::Resp, IpcError> {
        match req {
            WorldRequest::Hello { name } => {
                let ret = self.service.hello(name);
                Ok(WorldResponse::Hello(ret))
            }
        }
    }
}
#[derive(Serialize, Deserialize)]
enum WorldRequest {
    Hello { name: String },
}

#[derive(Serialize, Deserialize)]
enum WorldResponse {
    Hello(Result<String, u64>),
}

struct WorldClient {
    channel: Channel,
}

impl WorldClient {
    fn new(read: Pipe, write: Pipe) -> Self {
        Self {
            channel: Channel::new(read, write),
        }
    }
}

impl WorldClient {
    fn hello(&mut self, name: String) -> Result<String, u64> {
        let request = WorldRequest::Hello { name };
        let resp: Result<_, IpcError> = self
            .channel
            .call::<_, WorldResponse>("World.hello", request);
        match resp {
            Ok(WorldResponse::Hello(ret)) => ret,
            Err(e) => {
                panic!("IPC error: {:?}", e);
            }
        }
    }
}
// ---------------------------------
// end of auto generated code
// ---------------------------------

// the following code is written by users
struct WorldServer;

impl World for WorldServer {
    fn hello(&self, name: String) -> Result<String, u64> {
        if name == "error" {
            Err(1)
        } else {
            Ok(format!("hello, {}", name))
        }
    }
}

pub fn server_entry() -> Result<(), Error> {
    info!("server started");
    let fds = inherited_fds();
    assert_eq!(fds.len(), 2);
    let channel = Channel::new(fds[0].into(), fds[1].into());
    channel
        .execute(&mut WorldServer.server())
        .map_err(|_| Error::ServerError)?;
    Ok(())
}

pub fn client_entry() -> Result<(), Error> {
    info!("client started");

    // server can be spawned by any process which wants to start it.
    let (read_pipe, write_pipe) = spawn_server(
        0,
        Source::CellDep,
        &[CString::new("demo").unwrap().as_ref()],
    )
    .map_err(|_| Error::CkbSysError)?;

    let mut client = WorldClient::new(read_pipe.into(), write_pipe.into());
    let ret = client.hello("world".into()).unwrap();
    info!("IPC response: {:?}", ret);
    Ok(())
}

pub fn entry() -> Result<(), Error> {
    // enable logging by default
    drop(logger::init());

    let argv = argv();
    if !argv.is_empty() {
        server_entry()?;
    } else {
        client_entry()?;
    }
    Ok(())
}
