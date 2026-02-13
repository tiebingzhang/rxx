use anyhow::{Context, Result};
use rcgen::{CertificateParams, KeyPair};
use std::path::Path;

pub struct CertKeyPair {
    pub cert_pem: String,
    pub key_pem: String,
}

pub fn generate_self_signed_cert() -> Result<CertKeyPair> {
    println!("Generating self-signed certificate...");

    let mut params = CertificateParams::new(vec!["localhost".to_string()])
        .context("Failed to create certificate params")?;

    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "rxx-self-signed");

    let key_pair = KeyPair::generate().context("Failed to generate key pair")?;
    let cert = params
        .self_signed(&key_pair)
        .context("Failed to generate self-signed certificate")?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    println!("Self-signed certificate generated successfully");

    Ok(CertKeyPair { cert_pem, key_pem })
}

pub fn load_cert_from_file(cert_path: &Path, key_path: &Path) -> Result<CertKeyPair> {
    println!(
        "Loading certificate from {:?} and key from {:?}",
        cert_path, key_path
    );

    let cert_pem = std::fs::read_to_string(cert_path)
        .context(format!("Failed to read certificate from {:?}", cert_path))?;

    let key_pem = std::fs::read_to_string(key_path)
        .context(format!("Failed to read key from {:?}", key_path))?;

    println!("Certificate and key loaded successfully");

    Ok(CertKeyPair { cert_pem, key_pem })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_self_signed_cert() {
        let result = generate_self_signed_cert();
        assert!(result.is_ok());

        let cert_key = result.unwrap();
        assert!(!cert_key.cert_pem.is_empty());
        assert!(!cert_key.key_pem.is_empty());
        assert!(cert_key.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(cert_key.key_pem.contains("BEGIN PRIVATE KEY"));
    }
}
