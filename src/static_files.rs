use http_body_util::{Full, StreamBody, combinators::BoxBody, BodyExt};
use hyper::{Request, Response, Method, StatusCode};
use hyper::body::{Bytes, Frame};
use tokio_util::io::ReaderStream;
use tokio::fs::File;
use futures_util::TryStreamExt;

pub async fn serve(req: &Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, std::io::Error>>, std::io::Error>{
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
            let file = File::open("static/404.html").await;
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
    }
}