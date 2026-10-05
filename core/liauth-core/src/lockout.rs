//! Progressive delay applied after failed unlock attempts.
//!
//! The first two attempts are free; from there the wait grows quickly enough
//! to make brute force impractical while staying forgiving for typos.

/// Seconds the user must wait before the next attempt is accepted.
pub fn delay_seconds(failed_attempts: u32) -> u64 {
    match failed_attempts {
        0..=2 => 0,
        3 => 5,
        4 => 15,
        5 => 30,
        6 => 60,
        7 => 120,
        _ => 300,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_attempts_are_free() {
        assert_eq!(delay_seconds(0), 0);
        assert_eq!(delay_seconds(2), 0);
    }

    #[test]
    fn delay_grows_and_caps() {
        assert_eq!(delay_seconds(3), 5);
        assert!(delay_seconds(6) > delay_seconds(5));
        assert_eq!(delay_seconds(50), 300);
    }
}
