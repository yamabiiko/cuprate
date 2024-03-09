use hyper::body::Incoming;
use hyper::body::Bytes;
use hyper::Response as HyperResponse;
use hyper::Request as HyperRequest;
use crate::Response;
use crate::Request;
use tower::Service;
use std::pin::Pin;
use std::future::Future;
use crate::error::ErrorObject;
use crate::id::Id;
use crate::service::RpcService;
use std::collections::HashMap;
use std::any::Any;
use serde_json::value::RawValue;

use http_body_util::{combinators::BoxBody, BodyExt, Full};
use futures::FutureExt;

pub struct Server {
    rpc_server: RpcService
}

impl hyper::service::Service<HyperRequest<Incoming>> for Server
{
    type Response = HyperResponse<BoxBody<Bytes, hyper::Error>>;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: HyperRequest<Incoming>) -> Self::Future {
        let mut rpc_service = self.rpc_server.clone();
        async move {
            let response = match req.method() {
                &hyper::Method::POST => {
                    let body = req.collect().await.expect("Could not parse body of request").to_bytes();
                    let d: Request<&RawValue, &RawValue> = serde_json::from_slice(&body).unwrap();

                    let response = rpc_service.call(d).await.unwrap();
                    let s: String = serde_json::to_string(&response).unwrap();
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

#[cfg(test)]
mod test {
    use super::*;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
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
                let service = Server { rpc_server: RpcService };

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    println!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
}
