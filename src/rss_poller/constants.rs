use chrono::Duration;
use std::time::Duration as StdDuration;

const MINUTE_S: i64 = 60;
const HOUR_S: i64 = 3600;

/// The duration between scanning a given RSS feed
pub static POLL_INTERVAL: Duration = Duration::new(HOUR_S * 1, 0).unwrap();

/// The duration between making requests to WorldAnvil
pub static REQUEST_INTERVAL_STD: StdDuration = StdDuration::new(5 * MINUTE_S as u64, 0);

// The duration between checking if feeds need to be polled
pub static LOOP_INTERVAL_STD: StdDuration = StdDuration::new(HOUR_S as u64, 0);
