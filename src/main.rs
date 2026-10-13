//! Main CLI entry point for rping.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = rping::Cli::parse_args();

    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::WARN
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);

    if let Err(err) = rping::run(cli).await {
        eprintln!("\x1b[1;31merro no rping:\x1b[0m {err}");
        std::process::exit(1);
    }
}
