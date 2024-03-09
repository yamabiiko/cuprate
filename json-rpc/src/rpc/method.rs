use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    TestMethod,
    SecondMethod
}


#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rpc {
    TestMethod(crate::param::TestMethod),
    SecondMethod,
}
