use anyhow::{bail, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Certificate {
    pub run_id: String,
    pub artifact_hash: String,
    pub producer: String,
    pub verifier: String,
    pub signature: String,
}

pub fn sign(
    run_id: &str,
    artifact: &[u8],
    producer: &str,
    verifier: &str,
    key: &[u8],
) -> Result<Certificate> {
    if producer == verifier {
        bail!("producer and verifier must differ")
    }
    let artifact_hash = digest(artifact);
    let signature = digest(
        [
            key,
            run_id.as_bytes(),
            artifact_hash.as_bytes(),
            producer.as_bytes(),
            verifier.as_bytes(),
        ]
        .concat()
        .as_slice(),
    );
    Ok(Certificate {
        run_id: run_id.into(),
        artifact_hash,
        producer: producer.into(),
        verifier: verifier.into(),
        signature,
    })
}

pub fn verify(cert: &Certificate, artifact: &[u8], key: &[u8]) -> Result<()> {
    if cert.producer == cert.verifier {
        bail!("producer/verifier identity violation")
    }
    let artifact_hash = digest(artifact);
    if cert.artifact_hash != artifact_hash {
        bail!("artifact hash mismatch")
    }
    let expected = digest(
        [
            key,
            cert.run_id.as_bytes(),
            cert.artifact_hash.as_bytes(),
            cert.producer.as_bytes(),
            cert.verifier.as_bytes(),
        ]
        .concat()
        .as_slice(),
    );
    if cert.signature != expected {
        bail!("certificate signature mismatch")
    }
    Ok(())
}

pub fn verify_file(cert_path: &Path, artifact_path: &Path, key: &[u8]) -> Result<()> {
    let cert: Certificate = serde_json::from_slice(&std::fs::read(cert_path)?)?;
    verify(&cert, &std::fs::read(artifact_path)?, key)
}
fn digest(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_certificate_roundtrip() {
        let c = sign("r", b"a", "producer", "verifier", b"k").unwrap();
        assert!(verify(&c, b"a", b"k").is_ok());
        assert!(verify(&c, b"b", b"k").is_err());
    }
}
