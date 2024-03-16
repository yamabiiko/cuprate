use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    TestMethod,
    GetBlockCount
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rpc {
    TestMethod(crate::param::TestMethod),
    GetBlockCount
}
