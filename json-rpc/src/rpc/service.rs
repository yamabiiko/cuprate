use crate::Response;
use crate::Request;
use crate::id::Id;
use tower::Service;
use std::future::Future;
use std::pin::Pin;
use crate::error::ErrorObject;
use futures::FutureExt;
use std::task::Poll;
use std::sync::Arc;
use std::borrow::Cow;
use serde::{Serialize,Deserialize};
use serde_json::value::RawValue;
use crate::method::{Method, Rpc};

macro_rules! call_with_params {

	($method:expr, $request:expr, $fn:expr, $param:ty) => {{
        let id = $request.id.map(|id| id.into_owned());

        let Some(Ok(param)) = $request.params
            .map(|params| serde_json::from_str::<$param>(params.get())) else {
			    return async move { Ok(Response::invalid_params(id)) }.boxed()
		};

		$fn(param, id).boxed()
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
        println!("{}", req.method);
        let Ok(method) = serde_json::from_str::<crate::method::Method>(&req.method.get()) else {
            return async move { Ok(Response::error(ErrorObject::invalid_request(), Some(Id::Num(1239)))) }.boxed();
        };

	    use crate::method::Method::*;

        match method {
            TestMethod => {
               call_with_params!(method, req, test, crate::param::TestMethod)
            },
            SecondMethod => async move {
                Ok(Response::error(ErrorObject::invalid_request(), Some(Id::Num(404))))
            }.boxed(),
        }
    }
}

async fn test(param: crate::param::TestMethod, id: std::option::Option<Id<'_>>) -> Result<Response<&'static RawValue>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    Ok(Response::error(ErrorObject::invalid_request(), Some(Id::Num(202))))
}

