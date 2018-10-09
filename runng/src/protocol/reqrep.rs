use aio::NngAio;
use ctx::NngCtx;
use futures::{sync::oneshot};
use msg::NngMsg;
use runng_sys::*;
use std::{rc::Rc};
use super::*;

pub struct Req0 {
    socket: NngSocket
}

pub struct Rep0 {
    socket: NngSocket
}


impl Req0 {
    pub fn open() -> NngResult<Self> {
        let open_func = |socket: &mut nng_socket| unsafe { nng_req0_open(socket) };
        let socket_create_func = |socket| Req0{ socket };
        open(open_func, socket_create_func)
    }
}

impl Rep0 {
    pub fn open() -> NngResult<Self> {
        open(|socket| unsafe { nng_rep0_open(socket) }, 
            |socket| Rep0{ socket }
        )
    }
}

#[derive(PartialEq)]
enum ReqRepState {
    Ready,
    Sending,
    Receiving,
}

type MsgFuture = oneshot::Receiver<NngMsg>;

pub trait AsyncReqRep {
    fn send(&mut self) -> MsgFuture;
}

pub struct AsyncReqRepContext {
    ctx: Option<NngCtx>,
    state: ReqRepState,
    sender: Option<oneshot::Sender<NngMsg>>
}

impl AsyncReqRepContext {
    fn new() -> Box<AsyncReqRepContext> {
        let ctx = AsyncReqRepContext {
            ctx: None,
            state: ReqRepState::Ready,
            sender: None,
        };
        Box::new(ctx)
    }
    fn init(&mut self, aio: Rc<NngAio>) -> NngResult<()> {
        let ctx = NngCtx::new(aio)?;
        self.ctx = Some(ctx);
        Ok(())
    }
}

impl AsyncReqRep for AsyncReqRepContext {
    fn send(&mut self) -> MsgFuture {
        if self.state != ReqRepState::Ready {
            panic!();
        }
        let (sender, receiver) = oneshot::channel::<NngMsg>();
        self.sender = Some(sender);
        unsafe {
            let aio = self.ctx.as_ref().unwrap().aio();
            let ctx = self.ctx.as_ref().unwrap().ctx();
            self.state = ReqRepState::Sending;

            let mut request: *mut nng_msg = std::ptr::null_mut();
            // TODO: check result != 0
            let res = nng_msg_alloc(&mut request, 0);
            nng_aio_set_msg(aio, request);

            nng_ctx_send(ctx, aio);
        }
        
        receiver
    }
}

pub trait AsyncReqRepSocket: Socket {
    fn create_async_context(self) -> NngResult<Box<AsyncReqRepContext>>;
}

impl Socket for Req0 {
    fn socket(&self) -> nng_socket {
        self.socket.socket()
    }
}
impl Socket for Rep0 {
    fn socket(&self) -> nng_socket {
        self.socket.socket()
    }
}

impl Dial for Req0 {}
impl SendMsg for Req0 {}
impl Listen for Rep0 {}
impl RecvMsg for Rep0 {}

extern fn callback(arg : *mut ::std::os::raw::c_void) {
    unsafe {
        println!("callback {:?}", arg);
        let ctx = &mut *(arg as *mut AsyncReqRepContext);
        let aionng = ctx.ctx.as_ref().unwrap().aio();
        let ctxnng = ctx.ctx.as_ref().unwrap().ctx();
        match ctx.state {
            ReqRepState::Ready => panic!(),
            ReqRepState::Sending => {
                let res = nng_aio_result(aionng);
                if res != 0 {
                    //TODO: destroy message and set error
                    ctx.state = ReqRepState::Ready;
                    return;
                }

                // TODO: remove this test code
                let msg = NngMsg::new().unwrap();
                let sender = std::mem::replace(&mut ctx.sender, None);
                sender.unwrap().send(msg).unwrap();

                ctx.state = ReqRepState::Receiving;
                nng_ctx_recv(ctxnng, aionng);
            },
            ReqRepState::Receiving => {
                let res = nng_aio_result(aionng);
                if res != 0 {
                    //TODO: set error
                    ctx.state = ReqRepState::Ready;
                    return;
                }
                let msg = nng_aio_get_msg(aionng);
                let msg = NngMsg::new_msg(msg);
                let sender = std::mem::replace(&mut ctx.sender, None);
                sender.unwrap().send(msg).unwrap();
                ctx.state = ReqRepState::Ready;
            },
        }
    }
}

impl AsyncReqRepSocket for Req0 {
    fn create_async_context(self) -> NngResult<Box<AsyncReqRepContext>> {
        let mut ctx = AsyncReqRepContext::new();
        // This mess is needed to convert Box<_> to c_void
        let ctx_ptr = ctx.as_mut() as *mut _ as *mut std::os::raw::c_void;
        let aio = NngAio::new(self.socket, callback, ctx_ptr)?;
        let aio = Rc::new(aio);
        ctx.init(aio.clone());
        Ok(ctx)
    }
}