use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use crate::channel::SecureChannel;
use crate::discovery::SERVICE_TYPE;
use crate::{generate_pairing_code, SyncError};

pub enum ReceiverEvent {
    Payload(Vec<u8>),
    Failed(SyncError),
}

pub struct Receiver {
    pub code: String,
    pub port: u16,
    events: mpsc::Receiver<ReceiverEvent>,
    stop: Arc<AtomicBool>,
    daemon: Option<ServiceDaemon>,
    service_fullname: Option<String>,
    thread: Option<JoinHandle<()>>,
}

impl Receiver {
    pub fn start(device_name: &str) -> Result<Self, SyncError> {
        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0))?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        let code = generate_pairing_code();

        let (daemon, service_fullname) = match advertise(device_name, port) {
            Ok((daemon, fullname)) => (Some(daemon), Some(fullname)),
            Err(_) => (None, None),
        };

        let (sender, events) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);
        let thread_code = code.clone();

        let thread = std::thread::spawn(move || loop {
            if stop_flag.load(Ordering::Relaxed) {
                return;
            }
            match listener.accept() {
                Ok((stream, _)) => {
                    let _ = stream.set_nonblocking(false);
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
                    let result = handle_session(stream, &thread_code);
                    let event = match result {
                        Ok(payload) => ReceiverEvent::Payload(payload),
                        Err(error) => ReceiverEvent::Failed(error),
                    };
                    let _ = sender.send(event);
                    return;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    let _ = sender.send(ReceiverEvent::Failed(SyncError::Io(e)));
                    return;
                }
            }
        });

        Ok(Self {
            code,
            port,
            events,
            stop,
            daemon,
            service_fullname,
            thread: Some(thread),
        })
    }

    pub fn poll(&self, timeout: Duration) -> Option<ReceiverEvent> {
        match self.events.recv_timeout(timeout) {
            Ok(event) => Some(event),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => None,
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let (Some(daemon), Some(fullname)) = (self.daemon.take(), self.service_fullname.take()) {
            let _ = daemon.unregister(&fullname);
            let _ = daemon.shutdown();
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.stop();
    }
}

fn advertise(device_name: &str, port: u16) -> Result<(ServiceDaemon, String), SyncError> {
    let daemon = ServiceDaemon::new().map_err(|e| SyncError::Discovery(e.to_string()))?;
    let sanitized: String = device_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
        .take(32)
        .collect();
    let instance = if sanitized.trim().is_empty() {
        "LiAuth"
    } else {
        sanitized.trim()
    };
    let hostname = format!("liauth-{port}.local.");
    let info = ServiceInfo::new(SERVICE_TYPE, instance, &hostname, (), port, None::<HashMapProps>)
        .map_err(|e| SyncError::Discovery(e.to_string()))?
        .enable_addr_auto();
    let fullname = info.get_fullname().to_string();
    daemon
        .register(info)
        .map_err(|e| SyncError::Discovery(e.to_string()))?;
    Ok((daemon, fullname))
}

type HashMapProps = std::collections::HashMap<String, String>;

fn handle_session(stream: std::net::TcpStream, code: &str) -> Result<Vec<u8>, SyncError> {
    let mut channel = SecureChannel::establish_receiver(stream, code)?;
    let payload = channel.receive()?;
    channel.send(b"ok")?;
    Ok(payload)
}
