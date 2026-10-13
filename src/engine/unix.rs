//! Unix / Linux ICMP engine implementation using datagram / raw ICMP sockets.

use std::net::IpAddr;
use std::time::Duration;

use crate::engine::{EchoReply, PingEngine};
use crate::error::PingError;

/// Unix/Linux ICMP ping engine.
pub struct UnixPingEngine {
    _target: IpAddr,
}

impl UnixPingEngine {
    /// Creates a new Unix ICMP engine.
    ///
    /// # Errors
    /// Returns `PingError::HandleCreationFailed` if ICMP socket initialization fails.
    pub fn new(target: IpAddr) -> Result<Self, PingError> {
        Ok(Self { _target: target })
    }
}

impl PingEngine for UnixPingEngine {
    fn send_probe(
        &mut self,
        _target: IpAddr,
        _seq: u16,
        _payload: &[u8],
        _timeout: Duration,
        _ttl: u8,
    ) -> Result<EchoReply, PingError> {
        // Platform implementation for Unix systems
        Err(PingError::OsError("Unix engine probe stub".into()))
    }
}
