use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Correction, in seconds, applied on top of the OS clock. Set from an SNTP
/// measurement so codes stay valid even when the device clock has drifted.
static TIME_OFFSET: AtomicI64 = AtomicI64::new(0);

pub fn unix_now() -> i64 {
    system_now() + time_offset()
}

/// The raw OS clock, without the drift correction.
pub fn system_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn time_offset() -> i64 {
    TIME_OFFSET.load(Ordering::Relaxed)
}

pub fn set_time_offset(seconds: i64) {
    TIME_OFFSET.store(seconds, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_shifts_unix_now() {
        set_time_offset(0);
        let base = unix_now();
        set_time_offset(120);
        let shifted = unix_now();
        set_time_offset(0);
        assert!((shifted - base - 120).abs() <= 1);
    }
}
