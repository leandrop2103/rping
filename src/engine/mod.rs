//! Cross-platform ICMP probing engine interface and factory.

use crate::error::PingError;
use std::net::IpAddr;
use std::time::Duration;

#[cfg(windows)]
pub mod windows;

#[cfg(unix)]
pub mod unix;

/// Status code for an ICMP echo response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EchoStatus {
    /// Probe succeeded with an echo reply.
    Success,
    /// Probe timed out without a reply.
    Timeout,
    /// Time To Live expired in transit.
    TtlExpired,
    /// Destination net, host, protocol, or port was unreachable.
    DestinationUnreachable(String),
    /// Other ICMP or OS status code.
    Other(String),
}

/// Metadata and timing for a received ICMP Echo Reply.
#[derive(Debug, Clone)]
pub struct EchoReply {
    /// Sequence number of the probe.
    pub sequence: u16,
    /// Round-trip time measured for the probe.
    pub rtt: Duration,
    /// Length of the reply payload in bytes.
    pub bytes: usize,
    /// Time To Live reported by the replying host.
    pub ttl: u8,
    /// IP address of the replying host.
    pub source: IpAddr,
    /// Status code of the response.
    pub status: EchoStatus,
}

/// Abstract ICMP engine for transmitting probes and receiving echo replies.
pub trait PingEngine: Send {
    /// Transmits a probe packet to the specified target.
    ///
    /// # Errors
    /// Returns `PingError` on network, permission, or OS failure.
    fn send_probe(
        &mut self,
        target: IpAddr,
        seq: u16,
        payload: &[u8],
        timeout: Duration,
        ttl: u8,
    ) -> Result<EchoReply, PingError>;
}

/// Factory function creating the platform-appropriate ping engine.
///
/// # Errors
/// Returns `PingError` if initial socket or handle creation fails.
pub fn create_engine(target: IpAddr) -> Result<Box<dyn PingEngine>, PingError> {
    #[cfg(windows)]
    {
        windows::WindowsPingEngine::new(target).map(|eng| Box::new(eng) as Box<dyn PingEngine>)
    }

    #[cfg(unix)]
    {
        unix::UnixPingEngine::new(target).map(|eng| Box::new(eng) as Box<dyn PingEngine>)
    }

    #[cfg(not(any(windows, unix)))]
    {
        Err(PingError::OsError("unsupported target platform".into()))
    }
}
