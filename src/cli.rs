//! Command-line argument parsing and configuration.

use clap::Parser;

/// Modern, high-performance ping utility with colored output and cross-platform support.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "rping",
    author = "Leandro",
    version,
    about = "High-performance cross-platform ping utility written in Rust",
    long_about = "A modern reimplementation of iputils ping in Rust for Windows and Linux.\nFeatures zero-alloc hot path, unprivileged ICMP on Windows, colored output, and full statistics."
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Destination hostname or IP address to ping
    #[arg(value_name = "DESTINATION")]
    pub destination: String,

    /// Number of `ECHO_REQUEST` packets to send
    #[arg(short = 'c', long = "count", value_name = "COUNT")]
    pub count: Option<u64>,

    /// Wait interval seconds between sending each packet
    #[arg(
        short = 'i',
        long = "interval",
        default_value_t = 1.0,
        value_name = "SECONDS"
    )]
    pub interval: f64,

    /// Number of data payload bytes to send
    #[arg(short = 's', long = "size", default_value_t = 56, value_name = "BYTES")]
    pub size: usize,

    /// IP Time to Live (TTL) hop count (1-255)
    #[arg(short = 't', long = "ttl", default_value_t = 64, value_name = "TTL")]
    pub ttl: u8,

    /// Time to wait for a response in seconds
    #[arg(
        short = 'W',
        long = "timeout",
        default_value_t = 1.0,
        value_name = "SECONDS"
    )]
    pub timeout: f64,

    /// Exit after deadline seconds regardless of packet count
    #[arg(short = 'w', long = "deadline", value_name = "SECONDS")]
    pub deadline: Option<f64>,

    /// Force IPv4 name resolution
    #[arg(short = '4', long = "ipv4", conflicts_with = "ipv6")]
    pub ipv4: bool,

    /// Force IPv6 name resolution
    #[arg(short = '6', long = "ipv6", conflicts_with = "ipv4")]
    pub ipv6: bool,

    /// Audible output (ring bell on reply)
    #[arg(short = 'a', long = "audible")]
    pub audible: bool,

    /// Quiet output; only display startup and final summary lines
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Verbose output showing diagnostic tracing
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long = "no-color")]
    pub no_color: bool,
}
