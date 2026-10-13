//! Native Windows ICMP engine using IP Helper API (`iphlpapi.dll`).
//!
//! Provides high-performance, unprivileged ICMP echo probing for both IPv4 and IPv6
//! on Windows systems without requiring administrative privileges or raw socket drivers.

use std::ffi::c_void;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;
use std::time::Duration;

use crate::engine::{EchoReply, EchoStatus, PingEngine};
use crate::error::PingError;

type Handle = *mut c_void;
const INVALID_HANDLE_VALUE: Handle = -1isize as Handle;

// Windows IP status codes
const IP_SUCCESS: u32 = 0;
const IP_STATUS_BASE: u32 = 11_000;
const IP_DEST_NET_UNREACHABLE: u32 = IP_STATUS_BASE + 2;
const IP_DEST_HOST_UNREACHABLE: u32 = IP_STATUS_BASE + 3;
const IP_DEST_PROT_UNREACHABLE: u32 = IP_STATUS_BASE + 4;
const IP_DEST_PORT_UNREACHABLE: u32 = IP_STATUS_BASE + 5;
const IP_REQ_TIMED_OUT: u32 = IP_STATUS_BASE + 10;
const IP_TTL_EXPIRED_TRANSIT: u32 = IP_STATUS_BASE + 13;
const IP_TTL_EXPIRED_REASSEM: u32 = IP_STATUS_BASE + 14;
const IP_GENERAL_FAILURE: u32 = IP_STATUS_BASE + 50;

const AF_INET6: u16 = 23;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct IpOptionInformation {
    ttl: u8,
    tos: u8,
    flags: u8,
    options_size: u8,
    options_data: *mut u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct IcmpEchoReply {
    address: u32,
    status: u32,
    round_trip_time: u32,
    data_size: u16,
    reserved: u16,
    data: *mut u8,
    options: IpOptionInformation,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(clippy::struct_field_names)]
struct SockAddrIn6 {
    sin6_family: u16,
    sin6_port: u16,
    sin6_flowinfo: u32,
    sin6_addr: [u8; 16],
    sin6_scope_id: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(clippy::struct_field_names)]
struct Ipv6AddressEx {
    sin6_port: u16,
    sin6_flowinfo: u32,
    sin6_addr: [u8; 16],
    sin6_scope_id: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Icmpv6EchoReply {
    address: Ipv6AddressEx,
    status: u32,
    round_trip_time: u32,
}

#[link(name = "iphlpapi")]
extern "system" {
    fn IcmpCreateFile() -> Handle;
    fn Icmp6CreateFile() -> Handle;
    fn IcmpCloseHandle(icmp_handle: Handle) -> i32;

    fn IcmpSendEcho(
        icmp_handle: Handle,
        destination_address: u32,
        request_data: *const u8,
        request_size: u16,
        request_options: *const IpOptionInformation,
        reply_buffer: *mut u8,
        reply_size: u32,
        timeout: u32,
    ) -> u32;

    fn Icmp6SendEcho2(
        icmp_handle: Handle,
        event: Handle,
        apc_routine: usize,
        apc_context: usize,
        source_address: *const SockAddrIn6,
        destination_address: *const SockAddrIn6,
        request_data: *const u8,
        request_size: u16,
        request_options: *const IpOptionInformation,
        reply_buffer: *mut u8,
        reply_size: u32,
        timeout: u32,
    ) -> u32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetLastError() -> u32;
}

/// Windows ICMP probe engine using the Windows IP Helper API.
pub struct WindowsPingEngine {
    handle: Handle,
    buffer: Vec<u8>,
}

// Safety: Windows ICMP handles are thread-safe and can be sent across thread boundaries.
unsafe impl Send for WindowsPingEngine {}

impl WindowsPingEngine {
    /// Creates a new Windows ICMP engine instance for the specified target IP version.
    ///
    /// # Errors
    /// Returns `PingError::HandleCreationFailed` if Windows fails to open the ICMP device handle.
    pub fn new(target: IpAddr) -> Result<Self, PingError> {
        let handle = unsafe {
            if target.is_ipv6() {
                Icmp6CreateFile()
            } else {
                IcmpCreateFile()
            }
        };

        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            let err = unsafe { GetLastError() };
            return Err(PingError::HandleCreationFailed(format!(
                "Windows ICMP file creation failed with error code {err}"
            )));
        }

        // Allocate a reusable reply buffer on heap once to avoid runtime allocations during ping loop.
        let buf_size = 4096;
        let buffer = vec![0u8; buf_size];

        Ok(Self { handle, buffer })
    }

    fn map_status(status: u32) -> EchoStatus {
        match status {
            IP_SUCCESS => EchoStatus::Success,
            IP_REQ_TIMED_OUT => EchoStatus::Timeout,
            IP_TTL_EXPIRED_TRANSIT | IP_TTL_EXPIRED_REASSEM => EchoStatus::TtlExpired,
            IP_DEST_NET_UNREACHABLE => {
                EchoStatus::DestinationUnreachable("Rede de destino inalcançável".into())
            }
            IP_DEST_HOST_UNREACHABLE => {
                EchoStatus::DestinationUnreachable("Host de destino inalcançável".into())
            }
            IP_DEST_PROT_UNREACHABLE => {
                EchoStatus::DestinationUnreachable("Protocolo de destino inalcançável".into())
            }
            IP_DEST_PORT_UNREACHABLE => {
                EchoStatus::DestinationUnreachable("Porta de destino inalcançável".into())
            }
            IP_GENERAL_FAILURE => EchoStatus::Other("Falha geral".into()),
            code => EchoStatus::Other(format!("Código de status {code}")),
        }
    }

    fn ping_ipv4(
        &mut self,
        target_v4: Ipv4Addr,
        seq: u16,
        payload: &[u8],
        timeout: Duration,
        ttl: u8,
    ) -> Result<EchoReply, PingError> {
        let ip_u32 = u32::from_ne_bytes(target_v4.octets());
        let timeout_ms = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
        let payload_len = u16::try_from(payload.len()).map_err(|_| PingError::PayloadTooLarge {
            requested: payload.len(),
            max: u16::MAX as usize,
        })?;

        let options = IpOptionInformation {
            ttl,
            tos: 0,
            flags: 0,
            options_size: 0,
            options_data: ptr::null_mut(),
        };

        let reply_buf_len = u32::try_from(self.buffer.len()).unwrap_or(4096);

        // Safety:
        // 1. `self.handle` is a verified valid handle created via `IcmpCreateFile`.
        // 2. `payload` is valid memory for `payload_len` bytes.
        // 3. `self.buffer` is sized >= sizeof(IcmpEchoReply) + payload_len + 256.
        let ret = unsafe {
            IcmpSendEcho(
                self.handle,
                ip_u32,
                payload.as_ptr(),
                payload_len,
                &raw const options,
                self.buffer.as_mut_ptr(),
                reply_buf_len,
                timeout_ms,
            )
        };

        if ret == 0 {
            let last_error = unsafe { GetLastError() };
            let status = Self::map_status(last_error);
            return match status {
                EchoStatus::Timeout => Err(PingError::Timeout(seq)),
                EchoStatus::TtlExpired => Err(PingError::TtlExpired(IpAddr::V4(target_v4))),
                EchoStatus::DestinationUnreachable(msg) => {
                    Err(PingError::DestinationUnreachable(msg))
                }
                _ => Err(PingError::OsError(format!(
                    "IcmpSendEcho error {last_error}"
                ))),
            };
        }

        // Parse reply struct from buffer
        // Safety: `self.buffer` holds at least one `IcmpEchoReply` structure as returned by `IcmpSendEcho`.
        let reply: IcmpEchoReply =
            unsafe { ptr::read_unaligned(self.buffer.as_ptr().cast::<IcmpEchoReply>()) };

        let status = Self::map_status(reply.status);
        if status != EchoStatus::Success {
            return match status {
                EchoStatus::Timeout => Err(PingError::Timeout(seq)),
                EchoStatus::TtlExpired => Err(PingError::TtlExpired(IpAddr::V4(target_v4))),
                EchoStatus::DestinationUnreachable(msg) => {
                    Err(PingError::DestinationUnreachable(msg))
                }
                _ => Err(PingError::OsError(format!(
                    "ICMP reply status {}",
                    reply.status
                ))),
            };
        }

        let from_ip = Ipv4Addr::from(reply.address.to_ne_bytes());
        let rtt = Duration::from_millis(u64::from(reply.round_trip_time));

        Ok(EchoReply {
            sequence: seq,
            rtt,
            bytes: reply.data_size as usize,
            ttl: reply.options.ttl,
            source: IpAddr::V4(from_ip),
            status: EchoStatus::Success,
        })
    }

    fn ping_ipv6(
        &mut self,
        target_v6: Ipv6Addr,
        seq: u16,
        payload: &[u8],
        timeout: Duration,
        ttl: u8,
    ) -> Result<EchoReply, PingError> {
        let dest_addr = SockAddrIn6 {
            sin6_family: AF_INET6,
            sin6_port: 0,
            sin6_flowinfo: 0,
            sin6_addr: target_v6.octets(),
            sin6_scope_id: 0,
        };

        let options = IpOptionInformation {
            ttl,
            tos: 0,
            flags: 0,
            options_size: 0,
            options_data: ptr::null_mut(),
        };

        let timeout_ms = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
        let payload_len = u16::try_from(payload.len()).map_err(|_| PingError::PayloadTooLarge {
            requested: payload.len(),
            max: u16::MAX as usize,
        })?;

        let reply_buf_len = u32::try_from(self.buffer.len()).unwrap_or(4096);

        // Safety:
        // 1. `self.handle` is valid and created via `Icmp6CreateFile`.
        // 2. `dest_addr` correctly sets `sin6_family = AF_INET6`.
        // 3. `self.buffer` satisfies the buffer size requirements.
        let ret = unsafe {
            Icmp6SendEcho2(
                self.handle,
                ptr::null_mut(),
                0,
                0,
                ptr::null(),
                &raw const dest_addr,
                payload.as_ptr(),
                payload_len,
                &raw const options,
                self.buffer.as_mut_ptr(),
                reply_buf_len,
                timeout_ms,
            )
        };

        if ret == 0 {
            let last_error = unsafe { GetLastError() };
            let status = Self::map_status(last_error);
            return match status {
                EchoStatus::Timeout => Err(PingError::Timeout(seq)),
                EchoStatus::TtlExpired => Err(PingError::TtlExpired(IpAddr::V6(target_v6))),
                EchoStatus::DestinationUnreachable(msg) => {
                    Err(PingError::DestinationUnreachable(msg))
                }
                _ => Err(PingError::OsError(format!(
                    "Icmp6SendEcho2 error {last_error}"
                ))),
            };
        }

        // Safety: read reply header
        let reply: Icmpv6EchoReply =
            unsafe { ptr::read_unaligned(self.buffer.as_ptr().cast::<Icmpv6EchoReply>()) };

        let status = Self::map_status(reply.status);
        if status != EchoStatus::Success {
            return match status {
                EchoStatus::Timeout => Err(PingError::Timeout(seq)),
                EchoStatus::TtlExpired => Err(PingError::TtlExpired(IpAddr::V6(target_v6))),
                EchoStatus::DestinationUnreachable(msg) => {
                    Err(PingError::DestinationUnreachable(msg))
                }
                _ => Err(PingError::OsError(format!(
                    "ICMPv6 reply status {}",
                    reply.status
                ))),
            };
        }

        let from_ip = Ipv6Addr::from(reply.address.sin6_addr);
        let rtt = Duration::from_millis(u64::from(reply.round_trip_time));

        Ok(EchoReply {
            sequence: seq,
            rtt,
            bytes: payload.len(),
            ttl,
            source: IpAddr::V6(from_ip),
            status: EchoStatus::Success,
        })
    }
}

impl PingEngine for WindowsPingEngine {
    fn send_probe(
        &mut self,
        target: IpAddr,
        seq: u16,
        payload: &[u8],
        timeout: Duration,
        ttl: u8,
    ) -> Result<EchoReply, PingError> {
        match target {
            IpAddr::V4(v4) => self.ping_ipv4(v4, seq, payload, timeout, ttl),
            IpAddr::V6(v6) => self.ping_ipv6(v6, seq, payload, timeout, ttl),
        }
    }
}

impl Drop for WindowsPingEngine {
    fn drop(&mut self) {
        if !self.handle.is_null() && self.handle != INVALID_HANDLE_VALUE {
            // Safety: self.handle is verified valid and not double-freed.
            unsafe {
                IcmpCloseHandle(self.handle);
            }
            self.handle = INVALID_HANDLE_VALUE;
        }
    }
}
