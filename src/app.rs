//! Application orchestrator coordinating asynchronous ping loops, DNS, and signal handling.

use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::time::Duration;
use tokio::signal;
use tokio::time::Instant;
use tracing::{debug, info};

use crate::cli::Cli;
use crate::display::Printer;
use crate::engine::create_engine;
use crate::error::{PingError, Result};
use crate::stats::PingStatistics;

/// Resolves the destination string to an `IpAddr`, respecting `-4` / `-6` constraints.
///
/// # Errors
/// Returns `PingError` if DNS resolution fails or no compatible address is found.
pub fn resolve_target(destination: &str, force_v4: bool, force_v6: bool) -> Result<IpAddr> {
    // Check if directly parsable as IP
    if let Ok(ip) = destination.parse::<IpAddr>() {
        if force_v4 && !ip.is_ipv4() {
            return Err(PingError::AddressFamilyMismatch(ip));
        }
        if force_v6 && !ip.is_ipv6() {
            return Err(PingError::AddressFamilyMismatch(ip));
        }
        return Ok(ip);
    }

    // Resolve via DNS with a dummy port (0)
    let host_with_port = format!("{destination}:0");
    let addrs: Vec<SocketAddr> = host_with_port
        .to_socket_addrs()
        .map_err(|err| PingError::HostResolutionFailed(destination.to_string(), err.to_string()))?
        .collect();

    if addrs.is_empty() {
        return Err(PingError::NoAddressFound(destination.to_string()));
    }

    // Filter by IPv4 / IPv6 preference
    for socket_addr in &addrs {
        let ip = socket_addr.ip();
        if force_v4 && ip.is_ipv4() {
            return Ok(ip);
        }
        if force_v6 && ip.is_ipv6() {
            return Ok(ip);
        }
    }

    if force_v4 || force_v6 {
        return Err(PingError::NoAddressFound(format!(
            "{destination} (no matching {} address)",
            if force_v4 { "IPv4" } else { "IPv6" }
        )));
    }

    // Default preference: pick IPv4 first for legacy compatibility, else IPv6
    for socket_addr in &addrs {
        if socket_addr.ip().is_ipv4() {
            return Ok(socket_addr.ip());
        }
    }

    addrs
        .first()
        .map(SocketAddr::ip)
        .ok_or_else(|| PingError::NoAddressFound(destination.to_string()))
}

/// Runs the ping session according to CLI configuration.
///
/// # Errors
/// Returns `PingError` if target cannot be resolved or engine cannot be initialized.
pub async fn run(cli: Cli) -> Result<()> {
    let resolved_ip = resolve_target(&cli.destination, cli.ipv4, cli.ipv6)?;
    info!(target = %cli.destination, ip = %resolved_ip, "Target resolved");

    let printer = Printer::new(cli.no_color, cli.quiet, cli.audible, cli.timestamp_mode());
    let mut stats = PingStatistics::new(&cli.destination);
    let mut engine = create_engine(resolved_ip)?;

    printer
        .print_header(&cli.destination, resolved_ip, cli.size)
        .map_err(PingError::Io)?;

    let payload = vec![0x42u8; cli.size];
    let interval = Duration::from_secs_f64(cli.interval.max(0.001));
    let timeout = Duration::from_secs_f64(cli.timeout.max(0.001));
    let deadline_instant = cli
        .deadline
        .map(|d| Instant::now() + Duration::from_secs_f64(d));

    let mut seq: u16 = 1;

    loop {
        if let Some(dl) = deadline_instant {
            if Instant::now() >= dl {
                debug!("Ping deadline reached");
                break;
            }
        }

        stats.record_sent();

        match engine.send_probe(resolved_ip, seq, &payload, timeout, cli.ttl) {
            Ok(reply) => {
                stats.record_reply(reply.rtt);
                printer
                    .print_reply(
                        reply.bytes,
                        reply.source,
                        reply.sequence,
                        reply.ttl,
                        reply.rtt,
                    )
                    .map_err(PingError::Io)?;
            }
            Err(PingError::Timeout(s)) => {
                printer.print_timeout(s).map_err(PingError::Io)?;
            }
            Err(PingError::TtlExpired(from)) => {
                printer
                    .print_error_reply(Some(from), seq, "Tempo de vida (TTL) expirado no trânsito")
                    .map_err(PingError::Io)?;
            }
            Err(PingError::DestinationUnreachable(reason)) => {
                printer
                    .print_error_reply(None, seq, &reason)
                    .map_err(PingError::Io)?;
            }
            Err(e) => {
                debug!(seq = seq, error = %e, "Probe returned error");
                printer
                    .print_error_reply(None, seq, &e.to_string())
                    .map_err(PingError::Io)?;
            }
        }

        if let Some(max_count) = cli.count {
            if u64::from(seq) >= max_count {
                break;
            }
        }

        seq = seq.wrapping_add(1);

        // Sleep for interval or until Ctrl+C is received
        tokio::select! {
            () = tokio::time::sleep(interval) => {},
            res = signal::ctrl_c() => {
                if res.is_ok() {
                    debug!("Received Ctrl+C interrupt");
                    break;
                }
            }
        }
    }

    printer
        .print_summary(&stats.summary())
        .map_err(PingError::Io)?;

    Ok(())
}
