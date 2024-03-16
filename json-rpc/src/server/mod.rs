use hyper::body::{Incoming, Bytes};
use hyper::{
    Response as HyperResponse,
    Request as HyperRequest,
    StatusCode,
    Error
};

use crate::{
    Response,
    Request,
    error::ErrorObject,
    id::Id,
    service::RpcService
};

use tower::Service;
use std::{
    pin::Pin,
    future::Future
};
use serde_json::value::RawValue;

use http_body_util::{combinators::BoxBody, BodyExt, Full};
use futures::FutureExt;

pub struct Server {
    rpc_server: RpcService
}

impl hyper::service::Service<HyperRequest<Incoming>> for Server
{
    type Response = HyperResponse<BoxBody<Bytes, Error>>;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: HyperRequest<Incoming>) -> Self::Future {
        let mut rpc_service = self.rpc_server.clone();
        async move {
            let response = match req.method() {
                &hyper::Method::POST => {
                    let body_bytes = match req.collect().await {
                        Ok(body) => body.to_bytes(),
                        Err(e) => {
                            let response = HyperResponse::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(into_full::<String>(format!("Failed to read request body: {}", e).into()))
                                .unwrap();
                            return Ok(response);
                        }
                    };

                    let request: Request<&RawValue, &RawValue> = serde_json::from_slice(&body_bytes).unwrap();
                    let response = rpc_service.call(request).await.unwrap();

                    let str_response: String = serde_json::to_string(&response).unwrap();
                    Ok(HyperResponse::new(into_full(str_response)))
                }
                _ => {
                    // Client didnt POST
                    let ok: Response<'static, String> = Response::error(ErrorObject::invalid_request(), Some(Id::Num(1234)));
                    let s: String = serde_json::to_string(&ok).unwrap();
                    Ok(HyperResponse::new(into_full(s)))
                }
            };
            response
        }.boxed()
    }
}

// A body that consists of a single chunk
fn into_full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
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
    async fn run_test() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {

        tracing_subscriber::fmt().init();
        let addr: SocketAddr = "127.0.0.1:1337".parse().unwrap();

        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("Listening on http://{}", addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                let service = Server { rpc_server: RpcService };

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    tracing::info!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
}
