use std::net::SocketAddr;

use http_body_util::{combinators::BoxBody};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

mod static_files;
mod proxy;

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
        return proxy::serve(&req).await;
    }

    static_files::serve(&req).await
}