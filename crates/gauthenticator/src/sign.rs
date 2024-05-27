use rustls::crypto::ring;
use std::io;

pub struct Signer {
    signer: Box<dyn rustls::sign::Signer>,
}

impl Signer {
    pub fn new(private_key: &str) -> Result<Self, std::io::Error> {
        let key = Self::decode_rsa_key(private_key)?;
        let signing_key = ring::sign::any_supported_type(&key.into())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        let signer = signing_key
            .choose_scheme(&[rustls::SignatureScheme::RSA_PKCS1_SHA256])
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "Couldn't choose signing scheme")
            })?;

        Ok(Self { signer })
    }

    /// Decode a PKCS8 formatted RSA key.
    fn decode_rsa_key(
        pem_pkcs8: &str,
    ) -> Result<rustls::pki_types::PrivatePkcs8KeyDer, std::io::Error> {
        let mut reader = io::BufReader::new(pem_pkcs8.as_bytes());
        let mut private_keys = rustls_pemfile::pkcs8_private_keys(&mut reader);
        match private_keys.nth(0) {
            Some(key) => key,
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Error reading key from PEM",
            )),
        }
    }

    pub fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, rustls::Error> {
        self.signer.sign(digest)
    }
}
