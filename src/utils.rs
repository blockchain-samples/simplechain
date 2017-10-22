extern crate time;

// get current timestamp in ms
pub fn get_current_timestamp() -> i64 {
    let now = time::get_time();

    (now.sec as i64 * 1000) + (now.nsec as i64 / 1000)
}
