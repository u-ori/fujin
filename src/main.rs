use std::net::SocketAddr;

use http_body_util::{Full, combinators::BoxBody, BodyExt};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod static_files;

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
    static_files::serve(&req).await
}