use json_rpc::{
    Request as JsonRequest,
    Response as JsonResponse,
    Id,
    error::ErrorObject
};
use tower::Service;
use std::future::Future;
use std::pin::Pin;
use futures::FutureExt;
use std::task::Poll;
use serde_json::value::{RawValue, Value};
use tokio::time::{sleep, Duration};
use std::time::Instant;
use hyper::body::Bytes;
use std::borrow::Cow;
use crate::method::*;

macro_rules! call_json_rpc {

    // first case if the method needs params
    ($method:ident, $request:expr, $self:expr, $param:ty) => {{
        let id = $request.id.map(|id| id.into_owned());

        let Some(Ok(param)) = $request.params
            .map(|params| serde_json::from_str::<$param>(params.get())) else {
            return async move { Ok(JsonResponse::invalid_params(id)) }.boxed()
        };

        async move {
            let result = $self.core_rpc.call(crate::method::Rpc::$method(param)).await;
            if let Ok(result) = result {
                let result = serde_json::to_value(&result).unwrap();
                return Ok(JsonResponse::result(Cow::Owned(result), id))
            } else {
                return Ok(JsonResponse::invalid_params(id))
            }
        }.boxed()
    }};
}

macro_rules! call_bin_rpc {

    // first case if the method needs params
    ($method:ident, $bin_request:expr, $self:expr, $param:ty) => {{

        let Ok(param) = epee_encoding::from_bytes::<$param, Bytes>(&mut $bin_request.params) else {
            todo!("");
        };

        async move {
            let result = $self.core_rpc.call(crate::method::Rpc::$method(param)).await;
            if let Ok(crate::method::Response::$method(inner)) = result {
                let result = epee_encoding::to_bytes(inner).unwrap();
                return Ok(Bytes::from(result.freeze()))
            } else {
                todo!("");
            }
        }.boxed()
    }};
}

#[derive(Default, Clone)]
pub struct MiddlewareRpc { 
    pub core_rpc: CoreRpc
}

impl<'a> Service<JsonRequest<'a, &RawValue, &RawValue>> for MiddlewareRpc
{
    type Response = JsonResponse<'static, Value>;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: JsonRequest<'a, &RawValue, &RawValue>) -> Self::Future {
        let Ok(method) = serde_json::from_str::<crate::method::Method>(&req.method.get()) else {
            return async move { Ok(JsonResponse::error(ErrorObject::invalid_request(), Some(Id::Num(12)))) }.boxed()
        };

	    use crate::method::Method::*;
        let mut this = self.clone();

        match method {
            TestMethod => call_json_rpc!(TestMethod, req, this, crate::param::TestMethod),
            GetBlockCount => call_json_rpc!(TestMethod, req, this, crate::param::TestMethod)
        }
    }
}

#[derive(Default, Clone)]
pub struct CoreRpc;

impl<'a> Service<Rpc> for CoreRpc
{
    type Response = crate::method::Response;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: crate::method::Rpc) -> Self::Future {
        return async move { Ok(crate::method::Response::TestMethod( TestResponse { test: 0 })) }.boxed()
    }

}

pub struct CoreRpcRequest<T> {
    pub method: Box<String>,
    pub params: T
}

// CoreRPC Binary request
impl<'a> Service<CoreRpcRequest<Bytes>> for MiddlewareRpc
{
    type Response = Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: CoreRpcRequest<Bytes>) -> Self::Future {
        //let Ok(method) = serde_json::from_str::<Method>(&req.method) else {
        //   panic!("")
        //};
        let method = TestMethod;

	    use crate::method::Method::*;
        let mut this = self.clone();

        match method {
            TestMethod => call_bin_rpc!(TestMethod, req, this, crate::param::TestMethod),
            GetBlockCount => call_bin_rpc!(TestMethod, req, this, crate::param::TestMethod),
        }
    }
}
