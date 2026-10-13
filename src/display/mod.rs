//! Terminal output and presentation formatting for ping.

pub mod style;

use crate::stats::StatsSummary;
use std::io::{self, Write};
use std::net::IpAddr;
use std::time::Duration;
use style::Style;

/// Terminal printer for ping events with styling support.
pub struct Printer {
    style: Style,
    quiet: bool,
    audible: bool,
}

impl Printer {
    /// Creates a new output printer.
    #[must_use]
    pub fn new(no_color: bool, quiet: bool, audible: bool) -> Self {
        Self {
            style: Style::new(no_color),
            quiet,
            audible,
        }
    }

    /// Emits audible bell tone if enabled.
    pub fn ring_bell(&self) {
        if self.audible {
            print!("\x07");
            let _ = io::stdout().flush();
        }
    }

    /// Prints the initial ping header.
    ///
    /// # Errors
    /// Returns `io::Error` if stdout writing fails.
    pub fn print_header(
        &self,
        target_name: &str,
        resolved_ip: IpAddr,
        payload_bytes: usize,
    ) -> io::Result<()> {
        let total_bytes = payload_bytes + 28; // 20 IP + 8 ICMP
        let mut out = io::stdout().lock();

        let header = format!(
            "PING {target_name} ({resolved_ip}) {payload_bytes}({total_bytes}) bytes of data."
        );
        writeln!(out, "{}", self.style.bold_cyan(&header))?;
        out.flush()
    }

    /// Prints a successful echo reply.
    ///
    /// # Errors
    /// Returns `io::Error` if stdout writing fails.
    pub fn print_reply(
        &self,
        bytes: usize,
        from_ip: IpAddr,
        seq: u16,
        ttl: u8,
        rtt: Duration,
    ) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }

        self.ring_bell();

        let rtt_ms = rtt.as_secs_f64() * 1000.0;
        let colored_rtt = self.style.color_rtt(rtt_ms);

        let mut out = io::stdout().lock();
        writeln!(
            out,
            "{bytes} bytes from {from_ip}: icmp_seq={seq} ttl={ttl} time={colored_rtt}"
        )?;
        out.flush()
    }

    /// Prints a request timeout notification.
    ///
    /// # Errors
    /// Returns `io::Error` if stdout writing fails.
    pub fn print_timeout(&self, seq: u16) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }

        let msg = format!("Request timeout for icmp_seq {seq}");
        let mut out = io::stdout().lock();
        writeln!(out, "{}", self.style.bold_red(&msg))?;
        out.flush()
    }

    /// Prints an ICMP error message (e.g. TTL expired, Destination unreachable).
    ///
    /// # Errors
    /// Returns `io::Error` if stdout writing fails.
    pub fn print_error_reply(
        &self,
        from: Option<IpAddr>,
        seq: u16,
        message: &str,
    ) -> io::Result<()> {
        if self.quiet {
            return Ok(());
        }

        let prefix = from.map_or_else(
            || format!("icmp_seq={seq}: {message}"),
            |ip| format!("From {ip} icmp_seq={seq}: {message}"),
        );

        let mut out = io::stdout().lock();
        writeln!(out, "{}", self.style.bold_yellow(&prefix))?;
        out.flush()
    }

    /// Prints the final summary statistics block.
    ///
    /// # Errors
    /// Returns `io::Error` if stdout writing fails.
    pub fn print_summary(&self, summary: &StatsSummary) -> io::Result<()> {
        let mut out = io::stdout().lock();

        let header = format!("--- {} ping statistics ---", summary.target);
        writeln!(out, "\n{}", self.style.bold_cyan(&header))?;

        let total_ms = summary.total_duration.as_millis();
        let loss_colored = self.style.color_loss(summary.packet_loss_percent);

        writeln!(
            out,
            "{} packets transmitted, {} received, {} packet loss, time {}ms",
            summary.transmitted, summary.received, loss_colored, total_ms
        )?;

        if let (Some(min), Some(avg), Some(max), Some(mdev)) = (
            summary.rtt_min,
            summary.rtt_avg,
            summary.rtt_max,
            summary.rtt_mdev,
        ) {
            let stats_line =
                format!("rtt min/avg/max/mdev = {min:.3}/{avg:.3}/{max:.3}/{mdev:.3} ms");
            writeln!(out, "{}", self.style.bold(&stats_line))?;
        }

        out.flush()
    }
}
