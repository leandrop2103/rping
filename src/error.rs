//! Error types for the ping utility.

use std::net::IpAddr;
use thiserror::Error;

/// Core error type representing any failure during ping execution.
#[derive(Debug, Error)]
pub enum PingError {
    /// DNS or host name resolution failed.
    #[error("failed to resolve target '{0}': {1}")]
    HostResolutionFailed(String, String),

    /// No valid IP address was found for the target.
    #[error("no IP addresses found for target '{0}'")]
    NoAddressFound(String),

    /// Target address family does not match forced version (-4 or -6).
    #[error("resolved address '{0}' does not match requested IP version")]
    AddressFamilyMismatch(IpAddr),

    /// Failed to create the system ICMP handle or socket.
    #[error("failed to create ICMP handle/socket: {0}")]
    HandleCreationFailed(String),

    /// Probe request timed out.
    #[error("request timed out for icmp_seq {0}")]
    Timeout(u16),

    /// Time to live exceeded in transit.
    #[error("TTL expired in transit from {0}")]
    TtlExpired(IpAddr),

    /// Destination unreachable with specific cause.
    #[error("destination unreachable: {0}")]
    DestinationUnreachable(String),

    /// Payload size exceeds allowable maximum.
    #[error("payload size {requested} bytes exceeds maximum allowed ({max} bytes)")]
    PayloadTooLarge {
        /// Requested payload size in bytes.
        requested: usize,
        /// Maximum allowed payload size in bytes.
        max: usize,
    },

    /// An underlying I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Operating system specific error.
    #[error("OS error: {0}")]
    OsError(String),
}

/// Convenience type alias for Results returning [`PingError`].
pub type Result<T> = std::result::Result<T, PingError>;
