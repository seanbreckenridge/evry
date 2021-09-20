//! helper functions to deal with/describe time
use std::time::SystemTime;

/// gets the current time as milliseconds
pub fn epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("error getting unix timestamp")
        .as_millis()
}

// helper method; if the value (time) is not 0, append to the string buffer
#[doc(hidden)]
fn push_if_not_none(parts: &mut Vec<String>, time: u128, description: &str) {
    match time {
        0 => (),
        1 => parts.push(format!("{} {}", time, description)),
        _ => parts.push(format!("{} {}s", time, description)),
    }
}

/// convert milliseconds to human readable time,
/// used for debug output
///
/// Example:
///
/// Converts 4799805877 (time in milliseconds) to '55 days, 13 hours, 16 minutes, 45 seconds'
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
                push_if_not_none(&mut parts, days, "day");
            }
            push_if_not_none(&mut parts, hrs, "hour");
        }
        push_if_not_none(&mut parts, min, "minute");
    }
    push_if_not_none(&mut parts, sec, "second");
    parts.join(", ")
}
