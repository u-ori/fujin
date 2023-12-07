use http_body_util::{StreamBody, combinators::BoxBody, BodyExt};
use hyper::{Request, Response, Method, StatusCode};
use hyper::body::{Bytes, Frame};
use tokio_util::io::ReaderStream;
use tokio::fs::File;
use futures_util::TryStreamExt;

pub async fn serve(req: &Request<hyper::body::Incoming>) -> 
    Result<Response<BoxBody<Bytes, std::io::Error>>, std::io::Error> {
        let (filename, status) = match (req.method(), req.uri().path()) {
            (&Method::GET, "/") | (&Method::GET, "/index.html") => ("static/index.html", StatusCode::OK),
            (&Method::GET, "/debug") => ("static/debug.html", StatusCode::OK),
            _ => ("static/404.html", StatusCode::NOT_FOUND),
        };
        
        let file = File::open(filename).await;
        let file: File = file.unwrap();
        let reader_stream = ReaderStream::new(file);
        let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
        let boxed_body = stream_body.boxed();

        // Send response
        let response = Response::builder()
            .header("Content-Type", "text/html")
            .status(status)
            .body(boxed_body)
            .unwrap();

        Ok(response)
}