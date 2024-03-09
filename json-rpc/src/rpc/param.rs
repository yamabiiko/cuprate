use crate::Response;
use crate::Request;
use crate::error::ErrorObject;
use crate::id::Id;
use serde::{Deserialize, Serialize};


#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TestMethod {
    number: u8
}
