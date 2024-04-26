use crate::error::{Error, Result};
use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{io::ReadBuf, net::UdpSocket};

/// UdpServer represents an infinite series of requests over UDP.
///
/// It implements the `Iterator` trait, yielding successive `Request`s as they are received by the
/// server.
#[derive(Clone, Copy, Debug)]
pub struct UdpServer<'a> {
    /// The underlying UDP socket.
    socket: &'a UdpSocket,
}

/// A UDP request
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    /// The raw bytes of the request.
    pub body: Vec<u8>,
    /// The origin of the request.
    src_addr: SocketAddr,
}

impl<'a> futures::stream::Stream for UdpServer<'a> {
    type Item = std::io::Result<Request>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = [0; 512];
        let mut read_buf = ReadBuf::new(&mut buf);

        match futures::ready!(self.socket.poll_recv_from(cx, &mut read_buf)) {
            Ok(src_addr) => Poll::Ready(Some(Ok(Request {
                body: read_buf.filled().to_vec(),
                src_addr,
            }))),
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}

impl<'a> UdpServer<'a> {
    /// Returns a new `UdpServer` wrapping the given socket.
    pub fn new(socket: &'a UdpSocket) -> UdpServer {
        UdpServer { socket }
    }

    /// Reply to the given request with the given response over the server's socket.
    pub async fn reply(&self, request: &Request, response: &[u8]) -> Result<usize> {
        self.socket.send_to(response, request.src_addr).await.map_err(Error::from)
    }
}
