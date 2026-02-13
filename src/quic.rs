use anyhow::{Context, Result};
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::sync::Arc;

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
