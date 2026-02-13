use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use quinn::{ClientConfig, Connection, Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::cert::CertKeyPair;

pub fn create_server_config(cert_key: &CertKeyPair) -> Result<ServerConfig> {
    println!("Creating QUIC server configuration...");

    // Parse certificate
    let cert_der = rustls_pemfile::certs(&mut cert_key.cert_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to parse certificate")?;

    // Parse private key
    let key_der = rustls_pemfile::private_key(&mut cert_key.key_pem.as_bytes())
        .context("Failed to parse private key")?
        .context("No private key found")?;

    let cert_chain: Vec<CertificateDer> = cert_der;
    let private_key: PrivateKeyDer = key_der;

    let server_config = ServerConfig::with_single_cert(cert_chain, private_key)
        .context("Failed to create server config")?;

    println!("QUIC server configuration created");
    Ok(server_config)
}

pub fn create_client_config() -> Result<ClientConfig> {
    println!("Creating QUIC client configuration...");

    // Install default crypto provider
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Create a client config that skips certificate verification
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();

    let client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
            .context("Failed to create QUIC client config")?,
    ));

    println!("QUIC client configuration created (skipping cert verification)");
    Ok(client_config)
}

// Custom certificate verifier that skips all verification
#[derive(Debug)]
struct SkipServerVerification;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer,
        _intermediates: &[CertificateDer],
        _server_name: &rustls::pki_types::ServerName,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

pub async fn start_server(config: ServerConfig, bind_addr: SocketAddr) -> Result<Endpoint> {
    println!("Starting QUIC server on {}...", bind_addr);

    let endpoint =
        Endpoint::server(config, bind_addr).context("Failed to create QUIC server endpoint")?;

    println!("QUIC server started on {}", endpoint.local_addr()?);
    Ok(endpoint)
}

pub async fn connect_client(
    config: ClientConfig,
    bind_addr: SocketAddr,
    server_addr: SocketAddr,
) -> Result<quinn::Connection> {
    println!(
        "Starting QUIC client from {} to {}...",
        bind_addr, server_addr
    );

    let mut endpoint =
        Endpoint::client(bind_addr).context("Failed to create QUIC client endpoint")?;

    endpoint.set_default_client_config(config);

    let connection = endpoint
        .connect(server_addr, "localhost")
        .context("Failed to initiate connection")?
        .await
        .context("Failed to establish connection")?;

    println!("QUIC connection established to {}", server_addr);
    Ok(connection)
}

pub async fn send_file(connection: &Connection, file_path: &Path) -> Result<()> {
    println!("Opening file {:?} for sending...", file_path);

    let mut file = File::open(file_path)
        .await
        .context(format!("Failed to open file: {:?}", file_path))?;

    let metadata = file
        .metadata()
        .await
        .context("Failed to get file metadata")?;
    let file_size = metadata.len();

    if file_size == 0 {
        anyhow::bail!("Cannot send empty file");
    }

    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid file name")?;

    println!("Sending file: {} ({} bytes)", file_name, file_size);

    let (mut send, _recv) = connection
        .open_bi()
        .await
        .context("Failed to open bidirectional stream")?;

    // Send metadata: filename length (u32) + filename + file size (u64)
    send.write_u32(file_name.len() as u32)
        .await
        .context("Failed to send filename length")?;
    send.write_all(file_name.as_bytes())
        .await
        .context("Failed to send filename")?;
    send.write_u64(file_size)
        .await
        .context("Failed to send file size")?;

    println!("Metadata sent, streaming file content...");

    // Create progress bar
    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%)")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("Sending");

    // Stream file content and calculate hash
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024]; // 64KB chunks
    let mut total_sent = 0u64;

    loop {
        let n = file
            .read(&mut buffer)
            .await
            .context("Failed to read from file")?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);

        send.write_all(&buffer[..n])
            .await
            .context("Failed to send file chunk")?;

        total_sent += n as u64;
        pb.set_position(total_sent);
    }

    pb.finish_with_message("Sent");

    // Send hash
    let hash = hasher.finalize();
    send.write_all(&hash)
        .await
        .context("Failed to send file hash")?;

    send.finish().context("Failed to finish stream")?;
    println!(
        "File sent successfully: {} bytes (SHA256: {:x})",
        total_sent, hash
    );

    Ok(())
}

pub async fn receive_file(connection: &Connection, output_dir: &Path) -> Result<()> {
    println!("Waiting for incoming file stream...");

    let (_send, mut recv) = connection
        .accept_bi()
        .await
        .context("Failed to accept bidirectional stream")?;

    // Receive metadata
    let filename_len = recv
        .read_u32()
        .await
        .context("Failed to read filename length")?;

    let mut filename_bytes = vec![0u8; filename_len as usize];
    recv.read_exact(&mut filename_bytes)
        .await
        .context("Failed to read filename")?;

    let filename = String::from_utf8(filename_bytes).context("Invalid UTF-8 in filename")?;

    let file_size = recv.read_u64().await.context("Failed to read file size")?;

    println!("Receiving file: {} ({} bytes)", filename, file_size);

    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        tokio::fs::create_dir_all(output_dir)
            .await
            .context("Failed to create output directory")?;
        println!("Created output directory: {:?}", output_dir);
    }

    // Create output file
    let output_path = output_dir.join(&filename);
    let mut file = File::create(&output_path)
        .await
        .context(format!("Failed to create output file: {:?}", output_path))?;

    println!("Writing to {:?}...", output_path);

    // Create progress bar
    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%)")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("Receiving");

    // Receive file content and calculate hash
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024]; // 64KB chunks
    let mut total_received = 0u64;

    loop {
        match recv.read(&mut buffer).await {
            Ok(Some(n)) => {
                if total_received + n as u64 > file_size {
                    // This is the hash, not file data
                    let hash_start = (file_size - total_received) as usize;
                    if hash_start > 0 {
                        hasher.update(&buffer[..hash_start]);
                        file.write_all(&buffer[..hash_start])
                            .await
                            .context("Failed to write to file")?;
                        total_received += hash_start as u64;
                    }
                    pb.set_position(total_received);

                    // Read the hash
                    let mut received_hash = buffer[hash_start..n].to_vec();
                    let remaining = 32 - received_hash.len();
                    if remaining > 0 {
                        let mut hash_buf = vec![0u8; remaining];
                        recv.read_exact(&mut hash_buf)
                            .await
                            .context("Failed to read complete hash")?;
                        received_hash.extend_from_slice(&hash_buf);
                    }

                    pb.finish_with_message("Received");
                    file.flush().await.context("Failed to flush file")?;

                    // Verify hash
                    let computed_hash = hasher.finalize();
                    if computed_hash.as_slice() != received_hash.as_slice() {
                        anyhow::bail!(
                            "File integrity check failed: hash mismatch\nExpected: {:x}\nReceived: {}",
                            computed_hash,
                            hex::encode(&received_hash)
                        );
                    }

                    println!("File received successfully: {} bytes", total_received);
                    println!("Integrity verified (SHA256: {:x})", computed_hash);

                    if total_received != file_size {
                        anyhow::bail!(
                            "File size mismatch: expected {} bytes, received {} bytes",
                            file_size,
                            total_received
                        );
                    }

                    return Ok(());
                }

                hasher.update(&buffer[..n]);
                file.write_all(&buffer[..n])
                    .await
                    .context("Failed to write to file")?;

                total_received += n as u64;
                pb.set_position(total_received);
            }
            Ok(None) => {
                break;
            }
            Err(e) => {
                return Err(e).context("Failed to read from stream");
            }
        }
    }

    pb.finish_with_message("Received");
    file.flush().await.context("Failed to flush file")?;

    println!("File received successfully: {} bytes", total_received);

    if total_received != file_size {
        anyhow::bail!(
            "File size mismatch: expected {} bytes, received {} bytes",
            file_size,
            total_received
        );
    }

    Ok(())
}
