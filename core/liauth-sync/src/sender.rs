use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

use crate::channel::SecureChannel;
use crate::SyncError;

pub fn send_payload(addresses: &[IpAddr], port: u16, code: &str, payload: &[u8]) -> Result<(), SyncError> {
    let mut last_error = SyncError::Closed;
    for address in addresses {
        let stream =
            match TcpStream::connect_timeout(&SocketAddr::new(*address, port), Duration::from_secs(4)) {
                Ok(stream) => stream,
                Err(e) => {
                    last_error = SyncError::Io(e);
                    continue;
                }
            };
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        let mut channel = SecureChannel::establish_sender(stream, code)?;
        channel.send(payload)?;
        let ack = channel.receive()?;
        if ack != b"ok" {
            return Err(SyncError::PairingFailed);
        }
        return Ok(());
    }
    Err(last_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receiver::{Receiver, ReceiverEvent};
    use std::net::Ipv4Addr;

    #[test]
    fn end_to_end_transfer() {
        let receiver = Receiver::start("Test Device").unwrap();
        let code = receiver.code.clone();
        let port = receiver.port;

        let payload = b"encrypted backup bytes".to_vec();
        let sent = payload.clone();
        let handle =
            std::thread::spawn(move || send_payload(&[IpAddr::V4(Ipv4Addr::LOCALHOST)], port, &code, &sent));

        let event = receiver
            .poll(std::time::Duration::from_secs(15))
            .expect("receiver event");
        match event {
            ReceiverEvent::Payload(received) => assert_eq!(received, payload),
            ReceiverEvent::Failed(error) => panic!("receive failed: {error}"),
        }
        handle.join().unwrap().unwrap();
    }

    #[test]
    fn wrong_code_rejected() {
        let receiver = Receiver::start("Test Device").unwrap();
        let port = receiver.port;

        let handle = std::thread::spawn(move || {
            send_payload(&[IpAddr::V4(Ipv4Addr::LOCALHOST)], port, "000000", b"data")
        });

        let event = receiver
            .poll(std::time::Duration::from_secs(15))
            .expect("receiver event");
        let receiver_failed = matches!(event, ReceiverEvent::Failed(_));
        let sender_failed = handle.join().unwrap().is_err();
        assert!(receiver_failed || sender_failed);
    }

    #[test]
    fn unreachable_addresses_fall_through() {
        let receiver = Receiver::start("Test Device").unwrap();
        let code = receiver.code.clone();
        let port = receiver.port;

        let payload = b"payload".to_vec();
        let handle = std::thread::spawn(move || {
            send_payload(
                &[
                    IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)),
                    IpAddr::V4(Ipv4Addr::LOCALHOST),
                ],
                port,
                &code,
                &payload,
            )
        });

        let event = receiver
            .poll(std::time::Duration::from_secs(30))
            .expect("receiver event");
        assert!(matches!(event, ReceiverEvent::Payload(_)));
        handle.join().unwrap().unwrap();
    }
}
