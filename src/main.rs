use anyhow::Result;
use clap::{Parser, Subcommand};
use std::net::Ipv6Addr;
use std::path::PathBuf;

mod cert;
mod config;
mod db;
mod net;
mod peer;
mod quic;
mod server;
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

        /// Destination (IPv6 address or user ID)
        destination: String,

        /// Path to custom certificate file
        #[arg(long)]
        cert: Option<PathBuf>,

        /// Path to custom private key file
        #[arg(long)]
        key: Option<PathBuf>,
    },
    /// Receive a file from a remote peer
    Receive {
        /// Source (IPv6 address or user ID)
        source: String,

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
    /// Run the registration server
    Server {
        /// Database file path
        #[arg(long, default_value = "rxx.db")]
        db: String,

        /// Port to listen on
        #[arg(long, default_value = "3457")]
        port: u16,
    },
    /// Register user ID with the server
    Register {
        /// User ID to register (alphanumeric, max 20 chars)
        id: String,

        /// Server URL
        #[arg(long, default_value = "http://rxx.advistatech.com:3457")]
        server: String,
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

            let config = match config::Config::load() {
                Ok(cfg) => cfg,
                Err(_) => {
                    anyhow::bail!("Config file not found. Please run: rxx register <id>");
                }
            };

            let dest = destination.clone();
            let cfg = config.clone();
            let resolver = move || {
                let d = dest.clone();
                let c = cfg.clone();
                Box::pin(async move { net::resolve_peer(&d, &c).await })
                    as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Ipv6Addr>> + Send>>
            };

            // Perform UDP hole punching
            let peer_addr = udp::punch_hole(resolver, false).await?;

            // Create QUIC client config
            let client_config = quic::create_client_config()?;

            // Connect to QUIC server using the same port as UDP hole punching
            let bind_addr = format!("[::]:{}", udp::CLIENT_PORT).parse()?;
            let connection = quic::connect_client(client_config, bind_addr, peer_addr).await?;

            println!(
                "QUIC connection established to {}",
                connection.remote_address()
            );

            // Send file
            quic::send_file(&connection, &file).await?;

            // Close connection gracefully and wait for acknowledgment
            println!("DEBUG [MAIN]: Closing connection gracefully...");
            connection.close(0u32.into(), b"transfer complete");
            connection.closed().await;
            println!("DEBUG [MAIN]: Connection closed");

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

            let config = match config::Config::load() {
                Ok(cfg) => cfg,
                Err(_) => {
                    anyhow::bail!("Config file not found. Please run: rxx register <id>");
                }
            };

            let src = source.clone();
            let cfg = config.clone();
            let resolver = move || {
                let s = src.clone();
                let c = cfg.clone();
                Box::pin(async move { net::resolve_peer(&s, &c).await })
                    as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Ipv6Addr>> + Send>>
            };

            // Perform UDP hole punching
            let peer_addr = udp::punch_hole(resolver, true).await?;

            // Generate or load certificate
            let cert_key = if let (Some(cert_path), Some(key_path)) = (&cert, &key) {
                cert::load_cert_from_file(cert_path, key_path)?
            } else {
                cert::generate_self_signed_cert()?
            };

            // Create QUIC server config
            let server_config = quic::create_server_config(&cert_key)?;

            // Start QUIC server on the same port as UDP hole punching
            let bind_addr = format!("[::]:{}", udp::SERVER_PORT).parse()?;
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
        Commands::Server { db, port } => {
            println!("Starting server: db={}, port={}", db, port);
            server::run_server(&db, port).await?;
        }
        Commands::Register { id, server } => {
            if !id.chars().all(|c| c.is_alphanumeric()) {
                anyhow::bail!("ID must be alphanumeric only");
            }
            if id.len() > 20 {
                anyhow::bail!("ID must be 20 characters or less");
            }

            let ipv6 = net::get_local_ipv6()?;
            println!("Registering ID '{}' with IPv6 {}...", id, ipv6);

            let client = reqwest::Client::new();
            let response = client
                .post(format!("{}/register", server))
                .json(&serde_json::json!({
                    "id": id,
                    "ipv6": ipv6.to_string()
                }))
                .send()
                .await?;

            if response.status().is_success() {
                let config = config::Config {
                    user_id: id.clone(),
                    server_url: server.clone(),
                };
                config.save()?;
                println!("Successfully registered ID '{}'", id);
            } else if response.status() == reqwest::StatusCode::CONFLICT {
                anyhow::bail!("ID '{}' is already registered", id);
            } else {
                anyhow::bail!("Registration failed: {}", response.status());
            }
        }
    }

    Ok(())
}
