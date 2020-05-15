use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .collect()
}

/// Now with a fixed offset of the current system timezone
pub fn now() -> chrono::DateTime<chrono::offset::FixedOffset> {
    let date = chrono::offset::Local::now();
    date.with_timezone(&date.offset())
}
