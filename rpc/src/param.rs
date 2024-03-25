use epee_encoding::epee_object;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestMethod {
    number: u8,
}

epee_object!(
    TestMethod,
    number: u8,
);
