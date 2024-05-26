use crate::{CredentialsSchema, Error, GoogleCloudUserDirectory};
use serde::Deserialize;
use std::io::BufRead;

pub struct ProfileWithCredentials {
    pub credentials: CredentialsSchema,
    pub config: ProfileSchema,
}

#[derive(Deserialize)]
pub struct ProfileSchema {
    pub account: String,
    pub project: String,
}

/// need to load config map into this struct
pub struct GoogleCloudConfigurationContext {
    pub directory: GoogleCloudUserDirectory,
    pub profiles: Profiles,
}

impl GoogleCloudConfigurationContext {
    pub fn new() -> Result<Self, Error> {
        let dir = GoogleCloudUserDirectory::new()?;
        let profiles = Profiles::new(&dir)?;
        Ok(Self {
            directory: dir,
            profiles,
        })
    }
}

impl ProfileSchema {
    pub fn to_credentials(self) -> Result<ProfileWithCredentials, Error> {
        let dir = GoogleCloudUserDirectory::new()?;
        let credentials = dir.get_profile_adc(&self);
        let credentials = crate::credentials_from_file(credentials).credentials()?;
        Ok(ProfileWithCredentials {
            config: self,
            credentials,
        })
    }
}

pub struct Profiles {
    inner: std::collections::HashMap<String, toml::Table>,
}

impl Profiles {
    pub fn new(directory: &GoogleCloudUserDirectory) -> Result<Profiles, Error> {
        let mut profiles = Profiles {
            inner: std::collections::HashMap::<String, toml::Table>::new(),
        };

        let config = directory.get_config_default();
        let contents = std::fs::read_to_string(config.as_path())
            .map_err(|e| Error::FailedToLoad(format!("{} because {}", config.display(), e)))?;

        parse(&mut profiles, contents.as_ref());

        Ok(profiles)
    }

    pub fn get(&mut self, name: &str) -> Result<ProfileSchema, Error> {
        // read out the core profile
        let Some(core) = self.inner.remove(name) else {
            return Err(Error::ProfileNotFound(name.to_string()));
        };

        let profile: ProfileSchema = core
            .try_into()
            .map_err(|e| Error::InvalidProfile(e.to_string()))?;

        Ok(profile)
    }
}

pub fn parse(profiles: &mut Profiles, contents: &[u8]) {
    let mut lines = contents.lines();
    while let Some(line) = lines.next() {
        let Ok(line) = line else {
            continue;
        };

        let name = line.trim_matches(&['[', ']']);
        if name.is_empty() {
            continue;
        }

        // set name of profile and initialize table
        let name = name.to_string();
        let mut table = toml::Table::new();

        // add properties to the profile
        for property in lines.by_ref() {
            // if there is an error, skip to the next property
            let Ok(property) = property else {
                continue;
            };

            // an empty line indicates the end of the profile
            if property.is_empty() {
                break;
            }

            // read the property
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

        // add the profile to the profile config map
        profiles.inner.insert(name, table);
    }
}

#[cfg(test)]
mod test {
    use super::Profiles;
    use crate::GoogleCloudUserDirectory;

    #[test]
    pub fn deserializes_profiles() {
        let directory = GoogleCloudUserDirectory::new().unwrap();
        let profiles = Profiles::new(&directory);
        assert!(profiles.is_ok());
        
    }

    #[test]
    pub fn finds_adc() {
        let directory = GoogleCloudUserDirectory::new().unwrap();
        let mut profiles = Profiles::new(&directory).unwrap();

        let core_profile = profiles.get("core").unwrap();

        let credentials = directory.get_profile_adc(&core_profile);
        let credentials = crate::credentials_from_file(credentials);
        assert!(credentials.credentials().is_ok());

        
    }
}
