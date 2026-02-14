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

    let mut server_config = ServerConfig::with_single_cert(cert_chain, private_key)
        .context("Failed to create server config")?;

    let mut transport = quinn::TransportConfig::default();
    transport.max_idle_timeout(Some(quinn::IdleTimeout::from(quinn::VarInt::from_u32(
        300000,
    )))); // 5 minutes
    transport.keep_alive_interval(Some(std::time::Duration::from_secs(10)));
    transport.stream_receive_window(quinn::VarInt::from_u32(1024 * 1024)); // 1MB per stream
    transport.receive_window(quinn::VarInt::from_u64(10 * 1024 * 1024).unwrap()); // 10MB connection
    transport.send_window(10 * 1024 * 1024); // 10MB
    server_config.transport_config(Arc::new(transport));

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

    let mut client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
            .context("Failed to create QUIC client config")?,
    ));

    let mut transport = quinn::TransportConfig::default();
    transport.max_idle_timeout(Some(quinn::IdleTimeout::from(quinn::VarInt::from_u32(
        300000,
    )))); // 5 minutes
    transport.keep_alive_interval(Some(std::time::Duration::from_secs(10)));
    transport.stream_receive_window(quinn::VarInt::from_u32(1024 * 1024)); // 1MB per stream
    transport.receive_window(quinn::VarInt::from_u64(10 * 1024 * 1024).unwrap()); // 10MB connection
    transport.send_window(10 * 1024 * 1024); // 10MB
    client_config.transport_config(Arc::new(transport));

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

pub async fn send_file(connection: &Connection, file_path: &Path, user_id: &str) -> Result<()> {
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

    crate::debug!("DEBUG [SEND]: Opening bidirectional stream...");
    let (mut send, _recv) = connection
        .open_bi()
        .await
        .context("Failed to open bidirectional stream")?;
    crate::debug!("DEBUG [SEND]: Bidirectional stream opened");

    // Send metadata: user_id length (u32) + user_id + filename length (u32) + filename + file size (u64)
    crate::debug!("DEBUG [SEND]: Sending user_id length: {}", user_id.len());
    send.write_u32(user_id.len() as u32)
        .await
        .context("Failed to send user_id length")?;
    crate::debug!("DEBUG [SEND]: Sending user_id: {}", user_id);
    send.write_all(user_id.as_bytes())
        .await
        .context("Failed to send user_id")?;
    crate::debug!("DEBUG [SEND]: Sending filename length: {}", file_name.len());
    send.write_u32(file_name.len() as u32)
        .await
        .context("Failed to send filename length")?;
    crate::debug!("DEBUG [SEND]: Sending filename: {}", file_name);
    send.write_all(file_name.as_bytes())
        .await
        .context("Failed to send filename")?;
    crate::debug!("DEBUG [SEND]: Sending file size: {}", file_size);
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
    let mut chunk_count = 0u64;

    crate::debug!("DEBUG [SEND]: Starting file content loop...");
    loop {
        let n = file
            .read(&mut buffer)
            .await
            .context("Failed to read from file")?;

        println!(
            "DEBUG [SEND]: Read {} bytes from file (chunk #{}, total_sent={})",
            n, chunk_count, total_sent
        );

        if n == 0 {
            crate::debug!("DEBUG [SEND]: EOF reached, breaking loop");
            break;
        }

        hasher.update(&buffer[..n]);

        crate::debug!("DEBUG [SEND]: Calling write_all for {} bytes...", n);
        send.write_all(&buffer[..n])
            .await
            .context("Failed to send file chunk")?;
        crate::debug!("DEBUG [SEND]: write_all completed for {} bytes", n);

        total_sent += n as u64;
        chunk_count += 1;
        pb.set_position(total_sent);

        println!(
            "DEBUG [SEND]: Chunk #{} sent, total_sent={}/{}",
            chunk_count, total_sent, file_size
        );
    }

    println!(
        "DEBUG [SEND]: File content loop completed, total_sent={}",
        total_sent
    );
    pb.finish_with_message("Sent");

    // Send hash
    let hash = hasher.finalize();
    crate::debug!("DEBUG [SEND]: Sending SHA256 hash: {:x}", hash);
    send.write_all(&hash)
        .await
        .context("Failed to send file hash")?;
    crate::debug!("DEBUG [SEND]: Hash sent successfully");

    crate::debug!("DEBUG [SEND]: Calling send.finish()...");
    send.finish().context("Failed to finish stream")?;
    crate::debug!("DEBUG [SEND]: send.finish() completed");

    // Wait for the stream to be fully acknowledged
    crate::debug!("DEBUG [SEND]: Waiting for stream to be fully transmitted...");
    send.stopped().await.context("Stream was stopped by peer")?;
    crate::debug!("DEBUG [SEND]: Stream fully transmitted and acknowledged");

    println!(
        "File sent successfully: {} bytes (SHA256: {:x})",
        total_sent, hash
    );

    Ok(())
}

pub async fn receive_file(
    connection: &Connection,
    output_dir: &Path,
    user_folder: bool,
    config: &crate::config::Config,
) -> Result<()> {
    println!("Waiting for incoming file stream...");

    crate::debug!("DEBUG [RECV]: Calling accept_bi()...");
    let (_send, mut recv) = connection
        .accept_bi()
        .await
        .context("Failed to accept bidirectional stream")?;
    crate::debug!("DEBUG [RECV]: Bidirectional stream accepted");

    // Receive metadata
    crate::debug!("DEBUG [RECV]: Reading user_id length...");
    let user_id_len = recv
        .read_u32()
        .await
        .context("Failed to read user_id length")?;
    crate::debug!("DEBUG [RECV]: User_id length: {}", user_id_len);

    let mut user_id_bytes = vec![0u8; user_id_len as usize];
    crate::debug!("DEBUG [RECV]: Reading user_id bytes...");
    recv.read_exact(&mut user_id_bytes)
        .await
        .context("Failed to read user_id")?;

    let sender_id = String::from_utf8(user_id_bytes).context("Invalid UTF-8 in user_id")?;
    crate::debug!("DEBUG [RECV]: Sender ID: {}", sender_id);

    crate::debug!("DEBUG [RECV]: Reading filename length...");
    let filename_len = recv
        .read_u32()
        .await
        .context("Failed to read filename length")?;
    crate::debug!("DEBUG [RECV]: Filename length: {}", filename_len);

    let mut filename_bytes = vec![0u8; filename_len as usize];
    crate::debug!("DEBUG [RECV]: Reading filename bytes...");
    recv.read_exact(&mut filename_bytes)
        .await
        .context("Failed to read filename")?;

    let filename = String::from_utf8(filename_bytes).context("Invalid UTF-8 in filename")?;
    crate::debug!("DEBUG [RECV]: Filename: {}", filename);

    crate::debug!("DEBUG [RECV]: Reading file size...");
    let file_size = recv.read_u64().await.context("Failed to read file size")?;
    crate::debug!("DEBUG [RECV]: File size: {}", file_size);

    println!(
        "Receiving file: {} ({} bytes) from {}",
        filename, file_size, sender_id
    );

    // Determine output directory
    let final_output_dir = if user_folder {
        output_dir.join(&sender_id)
    } else {
        output_dir.to_path_buf()
    };

    // Create output directory if it doesn't exist
    if !final_output_dir.exists() {
        tokio::fs::create_dir_all(&final_output_dir)
            .await
            .context("Failed to create output directory")?;
        println!("Created output directory: {:?}", final_output_dir);
    }

    // Create output file
    let output_path = final_output_dir.join(&filename);
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
    let mut chunk_count = 0u64;

    crate::debug!("DEBUG [RECV]: Starting receive loop...");
    loop {
        println!(
            "DEBUG [RECV]: Calling recv.read() (chunk #{}, total_received={}/{})",
            chunk_count, total_received, file_size
        );
        match recv.read(&mut buffer).await {
            Ok(Some(n)) => {
                chunk_count += 1;
                crate::debug!("DEBUG [RECV]: recv.read() returned {} bytes (chunk #{}, total_received={}, file_size={})", n, chunk_count, total_received, file_size);

                if total_received + n as u64 > file_size {
                    println!(
                        "DEBUG [RECV]: Detected hash in this chunk (total would be {} > {})",
                        total_received + n as u64,
                        file_size
                    );
                    // This is the hash, not file data
                    let hash_start = (file_size - total_received) as usize;
                    println!(
                        "DEBUG [RECV]: hash_start={}, remaining_file_data={}",
                        hash_start, hash_start
                    );

                    if hash_start > 0 {
                        println!(
                            "DEBUG [RECV]: Writing final {} bytes of file data",
                            hash_start
                        );
                        hasher.update(&buffer[..hash_start]);
                        file.write_all(&buffer[..hash_start])
                            .await
                            .context("Failed to write to file")?;
                        total_received += hash_start as u64;
                    }
                    pb.set_position(total_received);

                    // Read the hash
                    let mut received_hash = buffer[hash_start..n].to_vec();
                    println!(
                        "DEBUG [RECV]: Extracted {} bytes of hash from current buffer",
                        received_hash.len()
                    );
                    let remaining = 32 - received_hash.len();
                    if remaining > 0 {
                        println!(
                            "DEBUG [RECV]: Need to read {} more bytes for complete hash",
                            remaining
                        );
                        let mut hash_buf = vec![0u8; remaining];
                        recv.read_exact(&mut hash_buf)
                            .await
                            .context("Failed to read complete hash")?;
                        received_hash.extend_from_slice(&hash_buf);
                        crate::debug!("DEBUG [RECV]: Complete hash received");
                    }

                    pb.finish_with_message("Received");
                    file.flush().await.context("Failed to flush file")?;

                    // Verify hash
                    let computed_hash = hasher.finalize();
                    crate::debug!("DEBUG [RECV]: Computed hash: {:x}", computed_hash);
                    println!(
                        "DEBUG [RECV]: Received hash: {}",
                        hex::encode(&received_hash)
                    );

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

                    config.execute_file_received_hook(&sender_id, &filename, file_size);

                    return Ok(());
                }

                crate::debug!("DEBUG [RECV]: Writing {} bytes to file", n);
                hasher.update(&buffer[..n]);
                file.write_all(&buffer[..n])
                    .await
                    .context("Failed to write to file")?;

                total_received += n as u64;
                pb.set_position(total_received);
                println!(
                    "DEBUG [RECV]: Chunk #{} written, total_received={}/{}",
                    chunk_count, total_received, file_size
                );
            }
            Ok(None) => {
                crate::debug!("DEBUG [RECV]: recv.read() returned None (stream finished), total_received={}, file_size={}", total_received, file_size);
                break;
            }
            Err(e) => {
                crate::debug!("DEBUG [RECV]: recv.read() returned error: {:?}", e);
                return Err(e).context("Failed to read from stream");
            }
        }
    }

    println!(
        "DEBUG [RECV]: Exited receive loop, total_received={}, file_size={}",
        total_received, file_size
    );
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
