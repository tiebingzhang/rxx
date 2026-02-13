use anyhow::Result;
use clap::{Parser, Subcommand};
use std::net::Ipv6Addr;
use std::path::PathBuf;

mod cert;
mod quic;
mod udp;

#[derive(Parser)]
#[command(name = "rxx")]
#[command(version, about = "UDP/IPv6 File Transfer Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a file to a remote peer
    Send {
        /// File to send
        file: PathBuf,

        /// Destination IPv6 address
        destination: Ipv6Addr,

        /// Path to custom certificate file
        #[arg(long)]
        cert: Option<PathBuf>,

        /// Path to custom private key file
        #[arg(long)]
        key: Option<PathBuf>,
    },
    /// Receive a file from a remote peer
    Receive {
        /// Source IPv6 address
        source: Ipv6Addr,

        /// Output directory for received file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Path to custom certificate file
        #[arg(long)]
        cert: Option<PathBuf>,

        /// Path to custom private key file
        #[arg(long)]
        key: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Send {
            file,
            destination,
            cert,
            key,
        } => {
            println!(
                "Send mode: file={:?}, destination={}, cert={:?}, key={:?}",
                file, destination, cert, key
            );

            // Perform UDP hole punching
            let peer_addr = udp::punch_hole(destination, false).await?;

            // Create QUIC client config
            let client_config = quic::create_client_config()?;

            // Connect to QUIC server
            let bind_addr = "[::]:0".parse()?;
            let connection = quic::connect_client(client_config, bind_addr, peer_addr).await?;

            println!(
                "QUIC connection established to {}",
                connection.remote_address()
            );

            // Send file
            quic::send_file(&connection, &file).await?;

            println!("File transfer completed successfully");
        }
        Commands::Receive {
            source,
            output,
            cert,
            key,
        } => {
            println!(
                "Receive mode: source={}, output={:?}, cert={:?}, key={:?}",
                source, output, cert, key
            );

            // Perform UDP hole punching
            let peer_addr = udp::punch_hole(source, true).await?;

            // Generate or load certificate
            let cert_key = if let (Some(cert_path), Some(key_path)) = (&cert, &key) {
                cert::load_cert_from_file(cert_path, key_path)?
            } else {
                cert::generate_self_signed_cert()?
            };

            // Create QUIC server config
            let server_config = quic::create_server_config(&cert_key)?;

            // Start QUIC server
            let bind_addr = "[::]:3458".parse()?;
            let endpoint = quic::start_server(server_config, bind_addr).await?;

            // Accept incoming connection
            println!("Waiting for QUIC connection from {}...", peer_addr);
            let incoming = endpoint
                .accept()
                .await
                .ok_or_else(|| anyhow::anyhow!("No incoming connection"))?;
            let connection = incoming.await?;

            println!(
                "QUIC connection accepted from {}",
                connection.remote_address()
            );

            // Receive file
            let output_path = output.unwrap_or_else(|| PathBuf::from("."));
            quic::receive_file(&connection, &output_path).await?;

            println!("File transfer completed successfully");
        }
    }

    Ok(())
}
