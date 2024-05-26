use crate::{CredentialsFile, CredentialsFileError, GoogleCloudUserDirectory};
use serde::Deserialize;
use std::io::BufRead;

pub struct ProfileWithCredentials {
    pub credentials: CredentialsFile,
    pub config: ProfileConfig,
}

#[derive(Deserialize)]
pub struct ProfileConfig {
    pub account: String,
    pub project: String,
}

impl ProfileConfig {
    pub fn new() -> Result<Self, CredentialsFileError> {
        let dir = GoogleCloudUserDirectory::new()?;
        let config = dir.get_config_default();
        let config = config.as_path();
        let contents = std::fs::read_to_string(config).map_err(|e| {
            CredentialsFileError::FailedToLoad(format!("{} because {}", config.display(), e))
        })?;
        let profile = ProfileConfig::parse(contents.as_ref()).unwrap();
        Ok(profile)
    }

    pub fn to_credentials(self) -> Result<ProfileWithCredentials, CredentialsFileError> {
        let dir = GoogleCloudUserDirectory::new()?;
        let credentials = dir.get_profile_adc(&self);
        let credentials = CredentialsFile::from_file(credentials)?;
        Ok(ProfileWithCredentials {
            config: self,
            credentials,
        })
    }
    pub fn parse(contents: &[u8]) -> Option<ProfileConfig> {
        let mut map = std::collections::HashMap::<String, toml::Table>::new();
        let mut lines = contents.lines();
        while let Some(line) = lines.next() {
            let Ok(line) = line else {
                continue;
            };

            let name = line.trim_matches(&['[', ']']);
            if name.is_empty() {
                continue;
            }
            let name = name.to_string();
            let mut table = toml::Table::new();

            while let Some(property) = lines.next() {
                let Ok(property) = property else {
                    continue;
                };

                if property.is_empty() {
                    break;
                }

                let mut parts = property.split('=');
                match (parts.next(), parts.next()) {
                    (Some(key), Some(value)) => {
                        table.insert(
                            key.trim().to_string(),
                            toml::Value::String(value.trim().to_string()),
                        );
                    }
                    _ => (),
                }
            }
            map.insert(name, table);
        }

        let core = map.remove("core")?;
        let profile: ProfileConfig = core.try_into().ok()?;
        Some(profile)
    }
}

#[cfg(test)]
mod test {
    use super::ProfileConfig;
    use crate::{CredentialsFile, GoogleCloudUserDirectory};

    #[test]
    pub fn deserializes_config() {
        let config = GoogleCloudUserDirectory::new()
            .unwrap()
            .get_config_default();

        let contents = std::fs::read_to_string(config).unwrap();
        let config = ProfileConfig::parse(contents.as_ref());

        assert!(config.is_some());

        ()
    }

    #[test]
    pub fn finds_adc() {
        let dir = GoogleCloudUserDirectory::new().unwrap();
        let config = dir.get_config_default();

        let contents = std::fs::read_to_string(config).unwrap();
        let profile = ProfileConfig::parse(contents.as_ref()).unwrap();

        let credentials = dir.get_profile_adc(&profile);
        let credentials = CredentialsFile::from_file(credentials);
        assert!(credentials.is_ok());

        ()
    }
}
