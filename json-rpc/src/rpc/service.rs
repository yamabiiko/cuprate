use crate::Response;
use crate::Request;
use crate::id::Id;
use tower::Service;
use std::future::Future;
use std::pin::Pin;
use crate::error::ErrorObject;
use futures::FutureExt;
use std::task::Poll;
use serde_json::value::RawValue;
use tokio::time::{sleep, Duration};
use std::time::Instant;

macro_rules! call_method {

    // first case if the method needs params
    ($method:expr, $request:expr, $fn:expr, $param:ty) => {{
        let id = $request.id.map(|id| id.into_owned());

        let Some(Ok(param)) = $request.params
            .map(|params| serde_json::from_str::<$param>(params.get())) else {
            return async move { Ok(Response::invalid_params(id)) }.boxed()
        };

        async move {
            let start_time = Instant::now();
            let result = $fn(param, id).await;
            let end_time = Instant::now();
            tracing::info!("Elapsed time for RPC method {:?} was {:?}", $method, end_time - start_time);
            result
        }.boxed()
    }};

    ($method:expr, $request:expr, $fn:expr) => {{
        let id = $request.id.map(|id| id.into_owned());

        async move {
            let start_time = Instant::now();
            let result = $fn(id).await;
            let end_time = Instant::now();
            tracing::info!("Elapsed time for RPC method {:?} was {:?}", $method, end_time - start_time);
            result
        }.boxed()
    }};
}

#[derive(Default, Clone)]
pub struct RpcService;

impl<'a> Service<Request<'a, &RawValue, &RawValue>> for RpcService
{
    type Response = Response<'static, &'static RawValue>;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<'a, &RawValue, &RawValue>) -> Self::Future {
        let Ok(method) = serde_json::from_str::<crate::method::Method>(&req.method.get()) else {
            let id = req.id.map(|id| id.into_owned());
            return async move { Ok(Response::error(ErrorObject::invalid_request(), id )) }.boxed();
        };

	    use crate::method::Method::*;

        match method {
            TestMethod => call_method!(method, req, test, crate::param::TestMethod),
            GetBlockCount => call_method!(method, req, get_block_count),
        }
    }
}

async fn test(param: crate::param::TestMethod, id: std::option::Option<Id<'_>>) -> Result<Response<&'static RawValue>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    sleep(Duration::from_secs(2)).await;
    Ok(Response::error(ErrorObject::invalid_request(), id))
}

async fn get_block_count(id: std::option::Option<Id<'_>>) -> Result<Response<&'static RawValue>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    Ok(Response::error(ErrorObject::invalid_request(), id))
}
