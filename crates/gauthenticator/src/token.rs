use crate::{
    credentials::{AuthorizedUserFile, ServiceAccountFile},
    sign, CredentialsFile,
};
use serde::Deserialize;

/// this is the audience (aud) in the JWT
const BIG_QUERY_AUTH_URL: &str = "https://bigquery.googleapis.com/";
//const GOOGLE_SEARCH_CONSOLE_AUTH_URL: &str = "https://searchconsole.googleapis.com/";
//const SITE_VERIFICATION_AUTH_URL: &str = "https://siteverification.googleapis.com/";

pub type TokenResult<T> = Result<T, TokenError>;

#[derive(thiserror::Error, Debug)]
pub enum TokenError {
    #[error("std::io\t{0:?}")]
    IO(#[from] std::io::Error),

    #[error("rustls\t{0:?}")]
    Rustls(#[from] rustls::Error),

    #[error("http\t{0:?}")]
    Http(String),
}

impl CredentialsFile {
    pub fn token(&self, audience: Option<String>) -> TokenResult<String> {
        match self {
            CredentialsFile::AuthorizedUser(user) => user.token(),
            CredentialsFile::ServiceAccount(service) => service.token(audience),
        }
    }
}

/// The response of a Service Account Key token exchange.
#[derive(Deserialize)]
#[allow(dead_code)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id_token: Option<String>,
    expires_in: i64,
}

impl AuthorizedUserFile {
    /// https://developers.google.com/identity/protocols/oauth2/web-server#httprest_2
    pub fn token(&self) -> TokenResult<String> {
        let result = ureq::post("https://oauth2.googleapis.com/token")
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", &self.refresh_token),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ]);
        let response = Self::handle_error(result)?;
        let data: TokenResponse = response.into_json()?;
        Ok(data.access_token)
    }

    fn handle_error(
        result: Result<ureq::Response, ureq::Error>,
    ) -> Result<ureq::Response, TokenError> {
        result.map_err(|e| {
            let error_header = format!("[{}] {}", e.kind(), e);
            let error = match &e.kind() {
                ureq::ErrorKind::HTTP => {
                    if let Some(response) = e.into_response() {
                        let http_header = format!(
                            "{} {} {}",
                            response.status(),
                            response.status_text(),
                            response.get_url()
                        );

                        let Ok(body) = response.into_string() else {
                            panic!("{}", http_header);
                        };

                        format!("{} {}", http_header, body)
                    } else {
                        error_header
                    }
                }
                _ => error_header,
            };

            TokenError::Http(error)
        })
    }
}

fn encode_base64<T: AsRef<[u8]>>(decoded: T) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::URL_SAFE.encode(decoded)
}

impl ServiceAccountFile {
    /// https://developers.google.com/identity/protocols/oauth2/service-account
    pub fn token(&self, audience: Option<String>) -> TokenResult<String> {
        let audience = audience.unwrap_or(BIG_QUERY_AUTH_URL.to_string());

        log::debug!("generating token for {audience}");

        //let pk = self.private_key().expect("failed to load private key");
        let signer = sign::Signer::new(&self.private_key)?;
        let (header, claims) =
            Self::jwt(&self.private_key_id, &self.client_email, audience.as_str());
        let jwt = format!("{}.{}", encode_base64(header), encode_base64(claims));
        let signature = encode_base64(signer.sign(jwt.as_bytes())?);
        let jwt = format!("{}.{}", jwt, signature);
        Ok(jwt)
    }

    /// <https://developers.google.com/identity/protocols/oauth2/service-account#jwt-auth>
    fn jwt(private_key_id: &str, client_email: &str, audience: &str) -> (String, String) {
        let iat = chrono::offset::Utc::now().timestamp();
        // sets to expire ~ 1hr from now, which is the max
        let expiry = iat + 3600 - 5;

        (
            // header
            serde_json::json!({
                "alg": "RS256",
                "typ": "JWT",
                "kid": private_key_id
            })
            .to_string(),
            // claims
            serde_json::json!({
                "iss": client_email,
                "sub": client_email,
                "aud": audience,
                "iat": iat,
                "exp": expiry
            })
            .to_string(),
        )
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_macros)]
    macro_rules! parse_json {
        ($($json:tt)+) => {
            ::serde_json::from_value(::serde_json::json!($($json)+)).expect("failed to deserialize")
        }
    }

    use crate::credentials::ServiceAccountFile;

    fn service_account_test() -> ServiceAccountFile {
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
    fn jwt_signs_without_error() {
        let sa = service_account_test();
        let token = sa.token(None).unwrap();
        assert!(!token.is_empty());
    }
}
