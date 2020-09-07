use std::time::SystemTime;

pub fn epoch_time() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("error getting unix timestamp")
        .as_millis()
}

