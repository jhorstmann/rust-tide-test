use tide::*;
use serde::*;
use serde::{Serialize, Deserialize};
use serde::ser::SerializeSeq;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::{Read, IoSliceMut, BufRead, Cursor};

#[derive(Debug, Serialize, Deserialize)]
struct QueryResponse {
    headers: Vec<String>,
    data: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryRequest {
    query: String
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    status: u16,
    message: String
}

impl ErrorBody {
    fn new(message: String) -> Self {
        ErrorBody::with_status(500, message)
    }

    fn with_status(status: u16, message: String) -> Self {
        Self {
            status,
            message
        }

    }
}

fn handle_error(e: impl std::error::Error) -> Response {
    let body = ErrorBody::with_status(400, e.to_string());

    Response::new(400)
        .body_json(&body)
        .unwrap_or_else(|_| Response::new(500))
}

macro_rules! try_or_400 {
    ( $expression:expr ) => {
        match $expression {
            Ok(x) => x,
            Err(e) => return handle_error(e)
        }
    }
}

struct VecReader {
    inner: io::Cursor<Vec<u8>>,
}

impl VecReader {
    fn new(data: Vec<u8>) -> Self {
        VecReader {
            inner: io::Cursor::new(data),
        }
    }
}

impl io::Read for VecReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl io::BufRead for VecReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }
    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }
}

impl futures_io::AsyncRead for VecReader {
    fn poll_read(mut self: Pin<&mut Self>, _: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>>
    {
        Poll::Ready(io::Read::read(&mut *self, buf))
    }

    fn poll_read_vectored(mut self: Pin<&mut Self>, _: &mut Context<'_>, bufs: &mut [IoSliceMut<'_>]) -> Poll<io::Result<usize>>
    {
        Poll::Ready(io::Read::read_vectored(&mut *self, bufs))
    }
}

impl futures_io::AsyncBufRead for VecReader {
    fn poll_fill_buf(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<&[u8]>>
    {
        Poll::Ready(io::BufRead::fill_buf(self.get_mut()))
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        io::BufRead::consume(self.get_mut(), amt)
    }
}


async fn sample_data(mut req: Request<()>) -> Response {

    let query : QueryRequest = try_or_400!(req.body_json().await);
    dbg!(query.query);

    let bytes: Vec<u8> = Vec::new();
    //let cursor = Cursor::new(bytes);

    let mut ser = serde_json::Serializer::new(bytes);
    let mut seq = ser.serialize_seq(Some(2)).unwrap();
    for i in 0..2 {
        seq.serialize_element(&i).unwrap();
    }
    seq.end().unwrap();


    Response::new(200).body(futures_util::io::Cursor::new(ser.into_inner()))
}

#[async_std::main]
async fn main()  {
    let mut app = tide::new();
    app.at("/").post(sample_data);

    app.listen("127.0.0.1:8080").await.unwrap();
}
