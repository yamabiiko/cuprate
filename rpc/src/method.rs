use serde::{Deserialize, Serialize};
use epee_encoding::epee_object;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    TestMethod,
    GetBlockCount
}


#[derive(Debug, Clone)]
pub enum Rpc {
    TestMethod(crate::param::TestMethod),
    GetBlockCount
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct BaseResponse {
    credits: u64,
    status: String,
    top_hash: String,
    untrusted: bool,
}

epee_object!(
    BaseResponse,
    credits: u64,
    status: String,
    top_hash: String,
    untrusted: bool,
);

#[derive(Clone, Serialize, Deserialize)]
pub enum Response {
    TestMethod(TestResponse)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestResponse {
    base: BaseResponse,
    pub test: u8
}

epee_object!(
    TestResponse,
    base: BaseResponse,
    test: u8,
);

//flatten:
//base: BaseResponse,
