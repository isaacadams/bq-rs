pub type BQAuthResult<T> = Result<T, BQAuthError>;

#[derive(thiserror::Error, Debug)]
pub enum BQAuthError {
    #[error("serde_json {0:?}")]
    Serde(#[from] serde_json::Error),

    #[error("reqwest {0:?}")]
    Reqwest(#[from] reqwest::Error),

    #[error("std::io {0:?}")]
    IO(#[from] std::io::Error),

    #[error("rustls {0:?}")]
    Rustls(#[from] rustls::Error),
}

/// this is the audience (aud) in the JWT
const BIG_QUERY_AUTH_URL: &str = "https://bigquery.googleapis.com/";

fn encode_base64<T: AsRef<[u8]>>(decoded: T) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::URL_SAFE.encode(decoded)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ServiceAccountKey {
    #[serde(rename = "type")]
    pub key_type: Option<String>,
    pub project_id: Option<String>,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: Option<String>,
    pub auth_uri: Option<String>,
    pub token_uri: Option<String>,
    pub auth_provider_x509_cert_url: Option<String>,
    pub client_x509_cert_url: Option<String>,
}

/// <https://developers.google.com/identity/protocols/oauth2/service-account#jwt-auth>
fn jwt(private_key_id: &str, client_email: &str, audience: &str) -> (String, String) {
    let iat = chrono::offset::Utc::now().timestamp();
    // sets to expire ~ 1hr from now, which is the max
    let expiry = iat + 3600 - 5;

    (
        format!(
            r#"{{
            "alg": "RS256",
            "typ": "JWT",
            "kid": "{private_key_id}"
        }}"#
        )
        .replace(' ', "")
        .replace('\n', ""),
        format!(
            r#"
        {{
            "iss": "{client_email}",
            "sub": "{client_email}",
            "aud": "{audience}",
            "iat": {iat},
            "exp": {expiry}
        }}"#
        )
        .replace(' ', "")
        .replace('\n', ""),
    )
}

impl ServiceAccountKey {
    pub fn load() -> Result<Self, serde_json::Error> {
        let sa = match dotenvy::var("GOOGLE_APPLICATION_CREDENTIALS")
            .or(std::fs::read_to_string("./key.json"))
        {
            Ok(sa) => sa,
            Err(e) => {
                panic!("service account could not be found in GOOGLE_APPLICATION_CREDENTIALS environment variable or in local storage: {}", e);
            }
        };

        serde_json::from_str(&sa)
    }

    pub fn jwt(&self, audience: &str) -> (String, String) {
        jwt(&self.private_key_id, &self.client_email, audience)
    }

    pub fn access_token(&self) -> BQAuthResult<String> {
        //let pk = self.private_key().expect("failed to load private key");
        let signer = Signer::new(&self.private_key)?;
        let (header, claims) = self.jwt(BIG_QUERY_AUTH_URL);
        let jwt = format!("{}.{}", encode_base64(header), encode_base64(claims));
        let signature = encode_base64(signer.sign(jwt.as_bytes())?);
        let jwt = format!("{}.{}", jwt, signature);
        Ok(jwt)
    }
}

struct Signer {
    signer: Box<dyn rustls::sign::Signer>,
}

impl Signer {
    fn new(private_key: &str) -> Result<Self, std::io::Error> {
        use rustls::sign::{self, SigningKey};
        use std::io;
        let key = Self::decode_rsa_key(private_key)?;
        let signing_key = sign::RsaSigningKey::new(&key)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Couldn't initialize signer"))?;
        let signer = signing_key
            .choose_scheme(&[rustls::SignatureScheme::RSA_PKCS1_SHA256])
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "Couldn't choose signing scheme")
            })?;
        Ok(Self { signer })
    }

    /// Decode a PKCS8 formatted RSA key.
    fn decode_rsa_key(pem_pkcs8: &str) -> Result<rustls::PrivateKey, std::io::Error> {
        use std::io;
        let private_keys = rustls_pemfile::pkcs8_private_keys(&mut pem_pkcs8.as_bytes());

        match private_keys {
            Ok(mut keys) if !keys.is_empty() => {
                keys.truncate(1);
                Ok(rustls::PrivateKey(keys.remove(0)))
            }
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Not enough private keys in PEM",
            )),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Error reading key from PEM",
            )),
        }
    }

    fn sign(&self, digest: &[u8]) -> BQAuthResult<Vec<u8>> {
        Ok(self.signer.sign(digest)?)
    }
}

macro_rules! parse_json {
    ($($json:tt)+) => {
        ::serde_json::from_value(::serde_json::json!($($json)+)).expect("failed to deserialize")
    }
}

#[cfg(test)]
mod test {
    use super::ServiceAccountKey;

    fn service_account_test() -> ServiceAccountKey {
        parse_json!({
            "type": "service_account",
            "project_id": "test",
            "private_key_id": "26de294916614a5ebdf7a065307ed3ea9941902b",
            "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQDemmylrvp1KcOn\n9yTAVVKPpnpYznvBvcAU8Qjwr2fSKylpn7FQI54wCk5VJVom0jHpAmhxDmNiP8yv\nHaqsef+87Oc0n1yZ71/IbeRcHZc2OBB33/LCFqf272kThyJo3qspEqhuAw0e8neg\nLQb4jpm9PsqR8IjOoAtXQSu3j0zkXemMYFy93PWHjVpPEUX16NGfsWH7oxspBHOk\n9JPGJL8VJdbiAoDSDgF0y9RjJY5I52UeHNhMsAkTYs6mIG4kKXt2+T9tAyHw8aho\nwmuytQAfydTflTfTG8abRtliF3nil2taAc5VB07dP1b4dVYy/9r6M8Z0z4XM7aP+\nNdn2TKm3AgMBAAECggEAWi54nqTlXcr2M5l535uRb5Xz0f+Q/pv3ceR2iT+ekXQf\n+mUSShOr9e1u76rKu5iDVNE/a7H3DGopa7ZamzZvp2PYhSacttZV2RbAIZtxU6th\n7JajPAM+t9klGh6wj4jKEcE30B3XVnbHhPJI9TCcUyFZoscuPXt0LLy/z8Uz0v4B\nd5JARwyxDMb53VXwukQ8nNY2jP7WtUig6zwE5lWBPFMbi8GwGkeGZOruAK5sPPwY\nGBAlfofKANI7xKx9UXhRwisB4+/XI1L0Q6xJySv9P+IAhDUI6z6kxR+WkyT/YpG3\nX9gSZJc7qEaxTIuDjtep9GTaoEqiGntjaFBRKoe+VQKBgQDzM1+Ii+REQqrGlUJo\nx7KiVNAIY/zggu866VyziU6h5wjpsoW+2Npv6Dv7nWvsvFodrwe50Y3IzKtquIal\nVd8aa50E72JNImtK/o5Nx6xK0VySjHX6cyKENxHRDnBmNfbALRM+vbD9zMD0lz2q\nmns/RwRGq3/98EqxP+nHgHSr9QKBgQDqUYsFAAfvfT4I75Glc9svRv8IsaemOm07\nW1LCwPnj1MWOhsTxpNF23YmCBupZGZPSBFQobgmHVjQ3AIo6I2ioV6A+G2Xq/JCF\nmzfbvZfqtbbd+nVgF9Jr1Ic5T4thQhAvDHGUN77BpjEqZCQLAnUWJx9x7e2xvuBl\n1A6XDwH/ewKBgQDv4hVyNyIR3nxaYjFd7tQZYHTOQenVffEAd9wzTtVbxuo4sRlR\nNM7JIRXBSvaATQzKSLHjLHqgvJi8LITLIlds1QbNLl4U3UVddJbiy3f7WGTqPFfG\nkLhUF4mgXpCpkMLxrcRU14Bz5vnQiDmQRM4ajS7/kfwue00BZpxuZxst3QKBgQCI\nRI3FhaQXyc0m4zPfdYYVc4NjqfVmfXoC1/REYHey4I1XetbT9Nb/+ow6ew0UbgSC\nUZQjwwJ1m1NYXU8FyovVwsfk9ogJ5YGiwYb1msfbbnv/keVq0c/Ed9+AG9th30qM\nIf93hAfClITpMz2mzXIMRQpLdmQSR4A2l+E4RjkSOwKBgQCB78AyIdIHSkDAnCxz\nupJjhxEhtQ88uoADxRoEga7H/2OFmmPsqfytU4+TWIdal4K+nBCBWRvAX1cU47vH\nJOlSOZI0gRKe0O4bRBQc8GXJn/ubhYSxI02IgkdGrIKpOb5GG10m85ZvqsXw3bKn\nRVHMD0ObF5iORjZUqD0yRitAdg==\n-----END PRIVATE KEY-----\n",
            "client_email": "sa@test.iam.gserviceaccount.com",
            "client_id": "102851967901799660408",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
            "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/yup-test-sa-1%40yup-test-243420.iam.gserviceaccount.com"
        })
    }

    #[test]
    fn jwt_is_constructed() {
        let sa = service_account_test();
        assert_eq!(sa.client_id, Some("102851967901799660408".to_string()));
        let (_, _) = sa.jwt(super::BIG_QUERY_AUTH_URL);
    }

    #[test]
    fn jwt_signs_without_error() {
        let sa = service_account_test();
        let token = sa.access_token().unwrap();
        assert!(!token.is_empty());
    }
}
