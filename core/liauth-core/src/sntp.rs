//! Minimal SNTP (RFC 4330) client used to measure clock drift.
//!
//! The measured offset is applied to code generation through
//! [`crate::time::set_time_offset`]; the OS clock itself is never touched.

use std::net::{ToSocketAddrs, UdpSocket};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::CoreError;

const NTP_PACKET_LEN: usize = 48;
/// Seconds between 1900-01-01 (NTP epoch) and 1970-01-01 (Unix epoch).
const NTP_UNIX_DELTA: u64 = 2_208_988_800;

pub const DEFAULT_SERVERS: [&str; 3] = [
    "pool.ntp.org:123",
    "time.google.com:123",
    "time.cloudflare.com:123",
];

/// Queries the given server and returns the clock offset in seconds:
/// the value to add to the local clock to obtain true time.
pub fn query_offset(server: &str, timeout: Duration) -> Result<i64, CoreError> {
    let addr = server
        .to_socket_addrs()
        .map_err(|e| CoreError::InvalidParameter(format!("cannot resolve {server}: {e}")))?
        .next()
        .ok_or_else(|| CoreError::InvalidParameter(format!("no address for {server}")))?;

    let socket = UdpSocket::bind(("0.0.0.0", 0)).map_err(|e| CoreError::InvalidParameter(e.to_string()))?;
    socket
        .set_read_timeout(Some(timeout))
        .map_err(|e| CoreError::InvalidParameter(e.to_string()))?;

    let mut request = [0u8; NTP_PACKET_LEN];
    request[0] = 0b0010_0011; // LI = 0, VN = 4, Mode = 3 (client)

    let t1 = unix_micros();
    socket
        .send_to(&request, addr)
        .map_err(|e| CoreError::InvalidParameter(e.to_string()))?;

    let mut response = [0u8; NTP_PACKET_LEN];
    let (len, _) = socket
        .recv_from(&mut response)
        .map_err(|e| CoreError::InvalidParameter(e.to_string()))?;
    let t4 = unix_micros();

    if len < NTP_PACKET_LEN {
        return Err(CoreError::InvalidParameter("short ntp response".into()));
    }
    let mode = response[0] & 0x07;
    if mode != 4 && mode != 5 {
        return Err(CoreError::InvalidParameter("unexpected ntp mode".into()));
    }
    let stratum = response[1];
    if stratum == 0 {
        return Err(CoreError::InvalidParameter("kiss-of-death ntp response".into()));
    }

    let t2 = ntp_timestamp_micros(&response[32..40]);
    let t3 = ntp_timestamp_micros(&response[40..48]);
    Ok(clock_offset_seconds(t1, t2, t3, t4))
}

/// Tries each server in turn and returns the first successful offset.
pub fn measure_offset(servers: &[&str], timeout: Duration) -> Result<i64, CoreError> {
    let mut last = CoreError::InvalidParameter("no ntp servers configured".into());
    for server in servers {
        match query_offset(server, timeout) {
            Ok(offset) => return Ok(offset),
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// Standard NTP clock offset: ((T2 - T1) + (T3 - T4)) / 2, in whole seconds.
fn clock_offset_seconds(t1: i64, t2: i64, t3: i64, t4: i64) -> i64 {
    let offset_micros = ((t2 - t1) + (t3 - t4)) / 2;
    // Round to the nearest second so a sub-second drift stays at zero.
    let half = 500_000;
    if offset_micros >= 0 {
        (offset_micros + half) / 1_000_000
    } else {
        (offset_micros - half) / 1_000_000
    }
}

fn unix_micros() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as i64)
        .unwrap_or(0)
}

/// Converts an 8-byte NTP timestamp (seconds.fraction, both u32) into
/// microseconds since the Unix epoch.
fn ntp_timestamp_micros(bytes: &[u8]) -> i64 {
    let seconds = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64;
    let fraction = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as u64;
    let unix_seconds = seconds.wrapping_sub(NTP_UNIX_DELTA) as i64;
    let micros = (fraction * 1_000_000) >> 32;
    unix_seconds * 1_000_000 + micros as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_formula_matches_rfc() {
        // Server clock exactly 10 s ahead, 1 s network delay each way.
        let t1 = 0;
        let t2 = 11_000_000;
        let t3 = 11_000_000;
        let t4 = 2_000_000;
        assert_eq!(clock_offset_seconds(t1, t2, t3, t4), 10);
    }

    #[test]
    fn offset_rounds_to_nearest_second() {
        assert_eq!(clock_offset_seconds(0, 400_000, 400_000, 0), 0);
        assert_eq!(clock_offset_seconds(0, 600_000, 600_000, 0), 1);
        assert_eq!(clock_offset_seconds(0, -600_000, -600_000, 0), -1);
    }

    #[test]
    fn ntp_timestamp_conversion() {
        // NTP seconds = delta + 100, fraction = 0.5.
        let seconds = (NTP_UNIX_DELTA + 100) as u32;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&seconds.to_be_bytes());
        bytes.extend_from_slice(&(u32::MAX / 2 + 1).to_be_bytes());
        let micros = ntp_timestamp_micros(&bytes);
        assert_eq!(micros, 100_500_000);
    }
}
