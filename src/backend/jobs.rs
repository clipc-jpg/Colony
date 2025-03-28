

use std::time::SystemTime;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId {
    pub order_time: UnixTime,
    pub uuidv4: Uuid,
}

impl JobId {
    pub fn new() -> Self {
        return Self { order_time: UnixTime::now(), uuidv4: Uuid::new_v4() };
    }
}

impl Default for JobId {
    fn default() -> Self { return Self::new(); }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnixTime {
    pub seconds: u64
}

impl UnixTime {
    pub fn from(t: SystemTime) -> UnixTime {
        return UnixTime { seconds: t.duration_since(SystemTime::UNIX_EPOCH).expect("System Time appears to be before 1970").as_secs() };
    }

    pub fn now() -> UnixTime {
        return UnixTime::from(SystemTime::now());
    }
}












