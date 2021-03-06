Rust high-level wrapper around [NNG](https://github.com/nanomsg/nng) (Nanomsg-Next-Gen):

> NNG, like its predecessors nanomsg (and to some extent ZeroMQ), is a lightweight, broker-less library, offering a simple API to solve common recurring messaging problems, such as publish/subscribe, RPC-style request/reply, or service discovery. The API frees the programmer from worrying about details like connection management, retries, and other common considerations, so that they can focus on the application instead of the plumbing.

Features:  
- Use [nng_aio](https://nanomsg.github.io/nng/man/v1.1.0/nng_aio.5) for asynchronous I/O
- Use [nng_ctx](https://nanomsg.github.io/nng/man/v1.1.0/nng_ctx.5) for advanced protocol handling
- Leverage [futures](https://docs.rs/futures) crate for ease of use with [tokio](https://tokio.rs/) and eventual support of [`async`/`await`](https://github.com/rust-lang/rust/issues/50547)

## Examples

Simple:
```rust
use runng::*;
fn test() -> Result<(), NngFail> {
    const url: &str = "inproc://test";
    let factory = Latest::default();
    let rep = factory.replier_open()?.listen(&url)?;
    let req = factory.requester_open()?.dial(&url)?;
    req.send(msg::NngMsg::new()?)?;
    rep.recv()?;
    Ok(())
}
```

Asynchronous I/O:
```rust
use futures::{
    future::Future,
    stream::Stream,
};
use runng::{
    *,
    protocol::{
        AsyncReply,
        AsyncRequest,
        AsyncSocket,
    },
};

fn aio() -> NngReturn {
    const url: &str = "inproc://test";

    let factory = Latest::default();
    let mut rep_ctx = factory
        .replier_open()?
        .listen(&url)?
        .create_async_context()?;

    let requester = factory.requester_open()?.dial(&url)?;
    let mut req_ctx = requester.create_async_context()?;
    let req_future = req_ctx.send(msg::NngMsg::new()?);
    rep_ctx.receive()
        .take(1).for_each(|_request|{
            let msg = msg::NngMsg::new().unwrap();
            rep_ctx.reply(msg).wait().unwrap();
            Ok(())
        }).wait();
    req_future.wait().unwrap()?;

    Ok(())
}
```

Additional examples [in `examples/` folder](https://github.com/jeikabu/runng/tree/master/runng/examples).
