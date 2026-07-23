use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::shared_types::state::SerialRuntime;


#[derive(Debug, Serialize,Deserialize , Clone)]
pub struct SerialParseError {
    pub raw: String,
    pub error: String,
}
pub struct SerialState {
    // pub port: Mutex<Option<Box<dyn SerialPort>>>,
    pub runtime: Mutex<Option<SerialRuntime>>,
}

pub const MAXBACTH:usize = 250;
pub const MAX_TIME_BETEEN:u128 = 33;