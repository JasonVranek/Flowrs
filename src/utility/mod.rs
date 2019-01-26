use std::time::{Duration, SystemTime};



pub fn get_time() -> Duration {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                         .expect("SystemTime::duration_since failed")
}