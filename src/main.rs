use std::net::SocketAddr;

use futures_util::TryStreamExt;
use http_body_util::{Full, StreamBody, combinators::BoxBody, BodyExt};
use hyper::body::{Bytes, Frame};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, Method, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, fs::File};
use tokio_util::io::ReaderStream;

static NOTFOUND: &[u8] = b"Not Found";
static NEEDTOPROXY: &[u8] = b"Need to proxy this request";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(service))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn service(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, std::io::Error>>, std::io::Error> {
    println!("{:#?}", req);
    if req.uri().path().starts_with("/fujin/") {
        println!("need to proxy this request");
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(NEEDTOPROXY.into()).map_err(|e| match e {}).boxed())
            .unwrap());
    }

    // TODO: Put this into static.rs
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            let file = File::open("static/index.html").await;
            let file: File = file.unwrap();
            let reader_stream = ReaderStream::new(file);
            let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
            let boxed_body = stream_body.boxed();

            // Send response
            let response = Response::builder()
                .header("Content-Type", "text/html")
                .status(StatusCode::OK)
                .body(boxed_body)
                .unwrap();

            Ok(response)
        },
        (&Method::GET, "/debug") => {
            let file = File::open("static/debug.html").await;
            let file: File = file.unwrap();
            let reader_stream = ReaderStream::new(file);
            let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
            let boxed_body = stream_body.boxed();

            // Send response
            let response = Response::builder()
                .header("Content-Type", "text/html")
                .status(StatusCode::OK)
                .body(boxed_body)
                .unwrap();

            Ok(response)
        },
        _ => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(NOTFOUND.into()).map_err(|e| match e {}).boxed())
                .unwrap())
        },
    }
}