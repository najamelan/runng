//! Request/reply pattern.

use super::*;
use runng_sys::*;

/// Request half of request/reply pattern.  See [nng_req](https://nanomsg.github.io/nng/man/v1.1.0/nng_req.7).
#[derive(Clone)]
pub struct Req0 {
    socket: NngSocket,
}

impl Req0 {
    /// Create a new request socket.  See [nng_req_open](https://nanomsg.github.io/nng/man/v1.1.0/nng_req_open.3).
    pub fn open() -> NngResult<Self> {
        let open_func = |socket: &mut nng_socket| unsafe { nng_req0_open(socket) };
        let socket_create_func = |socket| Req0 { socket };
        nng_open(open_func, socket_create_func)
    }
}

impl Socket for Req0 {
    fn socket(&self) -> &NngSocket {
        &self.socket
    }
    fn socket_mut(&mut self) -> &mut NngSocket {
        &mut self.socket
    }
}

impl Dial for Req0 {}
impl Listen for Req0 {}
impl SendMsg for Req0 {}
impl RecvMsg for Req0 {}

impl AsyncSocket for Req0 {
    type ContextType = AsyncRequestContext;
}
