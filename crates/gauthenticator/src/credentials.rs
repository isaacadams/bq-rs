use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::profile::ProfileConfig;

const ENV_VARIABLE_NAME: &str = "GOOGLE_APPLICATION_CREDENTIALS";

pub type CredentialFileResult = Result<CredentialsFile, CredentialsFileError>;

#[derive(thiserror::Error, Debug)]
pub enum CredentialsFileError {
    #[error("invalid credentials file because {0}")]
    InvalidCredentials(String),

    #[error("cannot load credentials from {0}")]
    FailedToLoad(String),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CredentialsFile {
    #[serde(rename = "authorized_user")]
    AuthorizedUser(AuthorizedUserFile),
    #[serde(rename = "service_account")]
    ServiceAccount(ServiceAccountFile),
}

impl CredentialsFile {
    pub fn project_id(&self) -> Option<&str> {
        match self {
            CredentialsFile::AuthorizedUser(_) => None,
            CredentialsFile::ServiceAccount(service) => service.project_id.as_deref(),
        }
    }

    pub fn email(&self) -> Option<&str> {
        match self {
            CredentialsFile::AuthorizedUser(_) => None,
            CredentialsFile::ServiceAccount(service) => Some(service.client_email.as_str()),
        }
    }

    pub fn deserialize(json: &str) -> Result<Self, CredentialsFileError> {
        serde_json::from_str(json)
            .map_err(|e| CredentialsFileError::InvalidCredentials(e.to_string()))
    }

    pub fn from_env<S: AsRef<str>>(env: S) -> CredentialFileResult {
        let contents = dotenvy::var(env.as_ref()).map_err(|e| {
            CredentialsFileError::FailedToLoad(format!("{} because {}", env.as_ref(), e))
        })?;
        log::debug!(
            "loading service account key contents from ${}",
            env.as_ref()
        );
        Self::deserialize(&contents)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> CredentialFileResult {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path).map_err(|e| {
            CredentialsFileError::FailedToLoad(format!("{} because {}", path.display(), e))
        })?;
        log::debug!(
            "loading service account key contents from {}",
            path.display()
        );
        Self::deserialize(&contents)
    }

    pub fn from_well_known_env() -> CredentialFileResult {
        Self::from_env(ENV_VARIABLE_NAME)
    }

    pub fn from_well_known_file() -> CredentialFileResult {
        let path = Self::build_well_known_file_path()?;
        Self::from_file(path)
    }

    fn build_well_known_file_path() -> Result<PathBuf, CredentialsFileError> {
        let mut path = Self::get_user_config_directory()?;
        path.push("gcloud");
        path.push("application_default_credentials.json");
        Ok(path)
    }

    fn get_user_config_directory() -> Result<PathBuf, CredentialsFileError> {
        let mut path = PathBuf::new();
        if cfg!(windows) {
            let app_data = std::env::var("APPDATA").map_err(|e| {
                CredentialsFileError::FailedToLoad(format!(
                    "environment variable $APPDATA because {}",
                    e
                ))
            })?;
            path.push(app_data);
        } else {
            let home = std::env::var("HOME").map_err(|e| {
                CredentialsFileError::FailedToLoad(format!(
                    "environment variable $HOME because {}",
                    e
                ))
            })?;
            path.push(home);
            path.push(".config");
        }
        Ok(path)
    }
}

pub struct GoogleCloudUserDirectory {
    root: PathBuf,
}

impl GoogleCloudUserDirectory {
    pub fn new() -> Result<Self, CredentialsFileError> {
        let mut config = Self::get_user_config_directory()?;
        config.push("gcloud");
        Ok(Self { root: config })
    }

    pub fn get_application_default_credentials(&self) -> PathBuf {
        let mut file = self.root.clone();
        file.push("application_default_credentials.json");
        file
    }

    pub fn get_config_default(&self) -> PathBuf {
        let mut file = self.root.clone();
        file.push("configurations");
        file.push("config_default");
        file
    }

    pub fn get_profile_adc(&self, profile: &ProfileConfig) -> PathBuf {
        let mut file = self.root.clone();
        file.push("legacy_credentials");
        file.push(&profile.account);
        file.push("adc.json");
        file
    }

    fn get_user_config_directory() -> Result<PathBuf, CredentialsFileError> {
        let mut path = PathBuf::new();
        if cfg!(windows) {
            let app_data = std::env::var("APPDATA").map_err(|e| {
                CredentialsFileError::FailedToLoad(format!(
                    "environment variable $APPDATA because {}",
                    e
                ))
            })?;
            path.push(app_data);
        } else {
            let home = std::env::var("HOME").map_err(|e| {
                CredentialsFileError::FailedToLoad(format!(
                    "environment variable $HOME because {}",
                    e
                ))
            })?;
            path.push(home);
            path.push(".config");
        }
        Ok(path)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizedUserFile {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceAccountFile {
    #[serde(rename = "type")]
    /// 1. authorized_user
    /// 2. service_account
    pub key_type: Option<String>,
    pub project_id: Option<String>,
    /// only present when `key_type` is `service_account`
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: Option<String>,
    pub auth_uri: Option<String>,
    pub token_uri: Option<String>,
    pub auth_provider_x509_cert_url: Option<String>,
    pub client_x509_cert_url: Option<String>,
}

#[cfg(test)]
mod test {
    use super::CredentialsFile;

    #[test]
    fn authorized_user_deserializes() {
        let json = serde_json::json!({
          "client_id": "12345.apps.googleusercontent.com",
          "client_secret": "d-12345",
          "refresh_token": "1//12345",
          "type": "authorized_user"
        });
        let credentials: CredentialsFile = serde_json::from_value(json).unwrap();
        let CredentialsFile::AuthorizedUser(authorized_user) = credentials else {
            panic!("failed to deserialize into property structure");
        };
        assert_eq!(
            authorized_user.client_id,
            "12345.apps.googleusercontent.com"
        );
        assert_eq!(authorized_user.client_secret, "d-12345");
        assert_eq!(authorized_user.refresh_token, "1//12345");
    }

    #[test]
    fn service_account_deserializes() {
        let json = serde_json::json!({
            "type": "service_account",
            "project_id": "test",
            "private_key_id": "26de294916614a5ebdf7a065307ed3ea9941902b",
            "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQDemmylrvp1KcOn\n9yTAVVKPpnpYznvBvcAU8Qjwr2fSKylpn7FQI54wCk5VJVom0jHpAmhxDmNiP8yv\nHaqsef+87Oc0n1yZ71/IbeRcHZc2OBB33/LCFqf272kThyJo3qspEqhuAw0e8neg\nLQb4jpm9PsqR8IjOoAtXQSu3j0zkXemMYFy93PWHjVpPEUX16NGfsWH7oxspBHOk\n9JPGJL8VJdbiAoDSDgF0y9RjJY5I52UeHNhMsAkTYs6mIG4kKXt2+T9tAyHw8aho\nwmuytQAfydTflTfTG8abRtliF3nil2taAc5VB07dP1b4dVYy/9r6M8Z0z4XM7aP+\nNdn2TKm3AgMBAAECggEAWi54nqTlXcr2M5l535uRb5Xz0f+Q/pv3ceR2iT+ekXQf\n+mUSShOr9e1u76rKu5iDVNE/a7H3DGopa7ZamzZvp2PYhSacttZV2RbAIZtxU6th\n7JajPAM+t9klGh6wj4jKEcE30B3XVnbHhPJI9TCcUyFZoscuPXt0LLy/z8Uz0v4B\nd5JARwyxDMb53VXwukQ8nNY2jP7WtUig6zwE5lWBPFMbi8GwGkeGZOruAK5sPPwY\nGBAlfofKANI7xKx9UXhRwisB4+/XI1L0Q6xJySv9P+IAhDUI6z6kxR+WkyT/YpG3\nX9gSZJc7qEaxTIuDjtep9GTaoEqiGntjaFBRKoe+VQKBgQDzM1+Ii+REQqrGlUJo\nx7KiVNAIY/zggu866VyziU6h5wjpsoW+2Npv6Dv7nWvsvFodrwe50Y3IzKtquIal\nVd8aa50E72JNImtK/o5Nx6xK0VySjHX6cyKENxHRDnBmNfbALRM+vbD9zMD0lz2q\nmns/RwRGq3/98EqxP+nHgHSr9QKBgQDqUYsFAAfvfT4I75Glc9svRv8IsaemOm07\nW1LCwPnj1MWOhsTxpNF23YmCBupZGZPSBFQobgmHVjQ3AIo6I2ioV6A+G2Xq/JCF\nmzfbvZfqtbbd+nVgF9Jr1Ic5T4thQhAvDHGUN77BpjEqZCQLAnUWJx9x7e2xvuBl\n1A6XDwH/ewKBgQDv4hVyNyIR3nxaYjFd7tQZYHTOQenVffEAd9wzTtVbxuo4sRlR\nNM7JIRXBSvaATQzKSLHjLHqgvJi8LITLIlds1QbNLl4U3UVddJbiy3f7WGTqPFfG\nkLhUF4mgXpCpkMLxrcRU14Bz5vnQiDmQRM4ajS7/kfwue00BZpxuZxst3QKBgQCI\nRI3FhaQXyc0m4zPfdYYVc4NjqfVmfXoC1/REYHey4I1XetbT9Nb/+ow6ew0UbgSC\nUZQjwwJ1m1NYXU8FyovVwsfk9ogJ5YGiwYb1msfbbnv/keVq0c/Ed9+AG9th30qM\nIf93hAfClITpMz2mzXIMRQpLdmQSR4A2l+E4RjkSOwKBgQCB78AyIdIHSkDAnCxz\nupJjhxEhtQ88uoADxRoEga7H/2OFmmPsqfytU4+TWIdal4K+nBCBWRvAX1cU47vH\nJOlSOZI0gRKe0O4bRBQc8GXJn/ubhYSxI02IgkdGrIKpOb5GG10m85ZvqsXw3bKn\nRVHMD0ObF5iORjZUqD0yRitAdg==\n-----END PRIVATE KEY-----\n",
            "client_email": "sa@test.iam.gserviceaccount.com",
            "client_id": "102851967901799660408",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
            "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/yup-test-sa-1%40yup-test-243420.iam.gserviceaccount.com"
        });
        let credentials: CredentialsFile = serde_json::from_value(json).unwrap();
        let CredentialsFile::ServiceAccount(service_account) = credentials else {
            panic!("failed to deserialize into property structure");
        };
        assert_eq!(
            service_account.private_key_id,
            "26de294916614a5ebdf7a065307ed3ea9941902b"
        );
        assert_eq!(service_account.client_id.unwrap(), "102851967901799660408");
        assert_eq!(
            service_account.client_email,
            "sa@test.iam.gserviceaccount.com"
        );
    }
}
