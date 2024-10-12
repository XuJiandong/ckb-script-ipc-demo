use alloc::{ffi::CString, format, string::String, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    env::argv,
    high_level::inherited_fds,
    log::info,
    logger,
    syscalls::{self, pipe},
};

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
use ckb_script_ipc_common::{channel::Channel, error::IpcError, ipc::Serve, pipe::Pipe};
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

    fn method(&self, req: &WorldRequest) -> Option<&'static str> {
        match req {
            WorldRequest::Hello { .. } => Some("World.hello"),
        }
    }
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

fn spawn_server() -> Result<(u64, u64), Error> {
    let (r1, w1) = match pipe() {
        Ok(v) => v,
        Err(_) => return Err(Error::CkbSysError),
    };
    let (r2, w2) = match pipe() {
        Ok(v) => v,
        Err(_) => return Err(Error::CkbSysError),
    };
    let inherited_fds = &[r2, w1];

    let arg1 = CString::new("demo").unwrap();
    let argv = &[arg1.as_c_str()];
    let argc = argv.len();
    let mut process_id: u64 = 0;
    let argv_ptr: Vec<*const i8> = argv.iter().map(|e| e.as_ptr()).collect();
    let mut spgs = syscalls::SpawnArgs {
        argc: argc as u64,
        argv: argv_ptr.as_ptr(),
        process_id: &mut process_id as *mut u64,
        inherited_fds: inherited_fds.as_ptr(),
    };
    // spawn itself
    syscalls::spawn(0, Source::CellDep, 0, 0, &mut spgs).map_err(|_| Error::CkbSysError)?;
    Ok((r1, w2))
}

pub fn client_entry() -> Result<(), Error> {
    info!("client started");

    let (read_pipe, write_pipe) = spawn_server()?;

    let mut client = WorldClient::new(read_pipe.into(), write_pipe.into());
    let ret = client.hello("world".into());
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
