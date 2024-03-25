use hyper::body::{Incoming, Bytes};
use hyper::{
    Response as HyperResponse,
    Request as HyperRequest,
    StatusCode,
    Error
};

use json_rpc::{
    Response,
    Request,
    error::ErrorObject,
    Id,
};

use std::{
    pin::Pin,
    future::Future
};

use http_body_util::{
    combinators::BoxBody,
    BodyExt, 
    Full
};

use serde_json::value::RawValue;

use futures::FutureExt;
use crate::core_service::MiddlewareRpc;
use tower::Service;

pub struct Server {
    rpc_server: MiddlewareRpc
}

impl hyper::service::Service<HyperRequest<Incoming>> for Server
{
    type Response = HyperResponse<BoxBody<Bytes, Error>>;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: HyperRequest<Incoming>) -> Self::Future {
        let mut rpc_service = self.rpc_server.clone();
        async move {
            let path = req.uri().path().to_owned();
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

            let response = match path.as_str() {
                "/json_rpc" => {
                    let request: Request<&RawValue, &RawValue> = serde_json::from_slice(&body_bytes).unwrap();
                    let response = rpc_service.call(request).await.unwrap();
                    Ok(HyperResponse::new(into_full(response.to_string())))
                },
                // binary core_rpc call here
                _ if path.ends_with(".bin") => {
                    todo!("")
                }
                // normal core_rpc call here
                _ => {
                    todo!("")
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
    use crate::core_service::CoreRpc;

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
                let service = Server { rpc_server: MiddlewareRpc { core_rpc: CoreRpc  } };

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    tracing::info!("Failed to serve connection: {:?}", err);
                }
            });
        }
    }
}
