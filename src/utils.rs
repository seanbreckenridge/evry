use std::time::SystemTime;

pub fn epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("error getting unix timestamp")
        .as_millis()
}

// if the value (time) is not 0, append to the string buffer
fn push_if_not_none(parts: &mut Vec<String>, time: u128, description: &str) {
    if time != 0 {
        parts.push(format!("{} {}", time, description));
    }
}

/// convert milliseconds to human readable time,
/// for debug output
pub fn describe_ms(ms: u128) -> String {
    let mut parts: Vec<String> = vec![];
    // convert to seconds to begin with
    let mut sec = ms / 1000;
    // if more than a minute
    if sec >= 60 {
        let mut min = sec / 60;
        sec %= 60;
        // if more than an hour
        if min >= 60 {
            let mut hrs = min / 60;
            min %= 60;
            // if more than a day
            if hrs >= 24 {
                let days = hrs / 24;
                hrs %= 24;
                push_if_not_none(&mut parts, days, "days");
            }
            push_if_not_none(&mut parts, hrs, "hours");
        }
        push_if_not_none(&mut parts, min, "minutes");
    }
    push_if_not_none(&mut parts, sec, "seconds");
    parts.join(", ")
}
