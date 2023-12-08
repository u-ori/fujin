use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use hyper::{Request, Response, StatusCode};
use hyper::body::Bytes;
use http_body_util::Full;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tokio::io;
use tokio::io::AsyncWriteExt;

pub async fn serve(req: &Request<hyper::body::Incoming>) -> 
    Result<Response<BoxBody<Bytes, std::io::Error>>, std::io::Error> {
        fetch(&req.uri().path()[7..req.uri().path().len()]).await.unwrap()
}

async fn fetch(req: &str) ->
    Result<Result<Response<BoxBody<Bytes, std::io::Error>>, std::io::Error>, Box<dyn std::error::Error + Send + Sync>> {
        let url: hyper::Uri = req.parse().unwrap();
        let host = url.host().expect("uri has no host");
        let port = url.port_u16().unwrap_or(80);
        let addr = format!("{}:{}", host, port);
        
        let stream = TcpStream::connect(addr).await?;
        let io = TokioIo::new(stream);

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let authority = url.authority().unwrap().clone();

        // Fetch the url...
        let req = Request::builder()
            .uri(url)
            .header(hyper::header::HOST, authority.as_str())
            .body(Empty::<Bytes>::new())?;

        let mut res = sender.send_request(req).await?;

        println!("Response: {}", res.status());
        println!("Headers: {:#?}\n", res.headers());

        let mut output: Vec<u8> = Vec::new();

        while let Some(next) = res.frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                output.extend(&chunk[..]);
            }
        }

        let mut response = Response::builder()
            .status(res.status());

        for header in res.headers().keys() {
            &response.headers_mut().unwrap().insert(header, res.headers().get(header).unwrap().clone());
        }

        let response = response.body(Full::new(output.into()).map_err(|e| match e {}).boxed());

        Ok(Ok(response.unwrap()))
}