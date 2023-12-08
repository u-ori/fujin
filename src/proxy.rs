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
        fetch().await;
        
        let need_to_proxy: &[u8] = b"Need to proxy this request";
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(need_to_proxy.into()).map_err(|e| match e {}).boxed())
            .unwrap());
}

async fn fetch() ->
    Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url: hyper::Uri = "http://example.com".parse().unwrap();
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
        // println!("Body: {:#?}\n", res.body());

        let mut output: Vec<u8> = Vec::new();

        while let Some(next) = res.frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                // io::stdout().write_all(&chunk).await?;
                output.extend(&chunk[..]);
            }
        }
        println!("{output:?}");


        Ok(())
}