use clap::Parser;
use ix_daemon::{DEFAULT_IDLE_TIMEOUT_MS, DEFAULT_SOCKET_PATH, Server};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "ixcheld", about = "Ixchel daemon for IPC and background sync")]
struct Args {
    #[arg(long, default_value = DEFAULT_SOCKET_PATH)]
    socket: String,

    #[arg(long, default_value_t = DEFAULT_IDLE_TIMEOUT_MS, help = "Idle timeout in milliseconds (0 to disable)")]
    idle_timeout: u64,

    #[arg(
        long,
        help = "Enable file watching for automatic sync on .ixchel/ changes"
    )]
    watch: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args = Args::parse();
    let server = Server::with_options(&args.socket, args.idle_timeout, args.watch);

    tracing::info!("Starting ixcheld with socket: {}", args.socket);

    if let Err(e) = server.run().await {
        tracing::error!("Server error: {}", e);
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}
