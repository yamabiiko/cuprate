use std::convert::Infallible;

use serde_json::Value;
use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::Response as HyperResponse;
use hyper::Request as HyperRequest;
use hyper::service::Service;
use crate::Response;
use crate::Request;
use std::pin::Pin;
use std::future::Future;
use crate::error::ErrorObject;
use crate::id::Id;

use hyper::body::Frame;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use futures::FutureExt;

pub struct HttpService;

pub struct RpcService;



impl Service<HyperRequest<Incoming>> for HttpService
{
    type Response = HyperResponse<BoxBody<Bytes, hyper::Error>>;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: HyperRequest<Incoming>) -> Self::Future {
        async move {
            let response = match req.method() {
                &hyper::Method::POST => {
                    let body = req.collect().await.expect("bruh").to_bytes();
                    let d: Value = serde_json::from_slice(&body).unwrap();
                    println!("{:?}", d);
                    // make call based on method
                    let method: Option<&str> = d.get("method").and_then(|val| val.as_str());
                    println!("{:?}", method);
                    let ok: Response<'static, String> = Response::error(ErrorObject::invalid_request(), Some(Id::Num(1234)));
                    let s: String = serde_json::to_string(&ok).unwrap();
                    Ok(HyperResponse::new(full(s)))
                }
                _ => {
                    // Client didnt POST
                    let ok: Response<'static, String> = Response::error(ErrorObject::invalid_request(), Some(Id::Num(1234)));
                    let s: String = serde_json::to_string(&ok).unwrap();
                    Ok(HyperResponse::new(full(s)))
                }
            };
            response
        }.boxed()
    }

}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

impl<'a, P, M> Service<Request<'a, M, P>> for RpcService
where
    M: Clone,
    P: Clone
{
    type Response = Response<'static, String>;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<'a, M, P>) -> Self::Future {
        async move {
            Ok(Response::error(ErrorObject::invalid_request(), Some(Id::Num(1234))))
        }.boxed()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::SocketAddr;
    use tokio::net::{TcpListener, TcpStream};
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    use hyper::server::conn::http1;

    #[tokio::test]
    async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let addr: SocketAddr = "127.0.0.1:1337".parse().unwrap();

        let listener = TcpListener::bind(&addr).await?;
        println!("Listening on http://{}", addr);
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                let service = HttpService;

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    println!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }

}
