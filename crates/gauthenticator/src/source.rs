use crate::{
    profile::{ProfileConfig, ProfileWithCredentials},
    CredentialsError, CredentialsSchema,
};

pub struct SourceDetails {
    project: String,
    email: String,
    source: Source,
}

pub enum Source {
    Normal(CredentialsSchema),
    Profile(ProfileWithCredentials),
}

impl Source {
    pub fn token(&self, audience: Option<String>) -> Result<String, crate::TokenError> {
        match self {
            Source::Normal(c) => c.token(audience),
            Source::Profile(p) => p.credentials.token(audience),
        }
    }

    pub fn project_id(&self) -> Option<&str> {
        match self {
            Source::Normal(c) => c.project_id(),
            Source::Profile(p) => Some(p.config.project.as_str()),
        }
    }

    pub fn email(&self) -> Option<&str> {
        match self {
            Source::Normal(c) => c.email(),
            Source::Profile(p) => Some(p.config.account.as_str()),
        }
    }

    pub fn load() -> Result<Source, CredentialsError> {
        if let Ok(source) = ProfileConfig::new()
            .and_then(|p| p.to_credentials())
            .map(|p| Source::Profile(p))
        {
            return Ok(source);
        }

        let credentials = CredentialsSchema::from_well_known_env()
            .or(CredentialsSchema::from_well_known_file())?;

        Ok(Source::Normal(credentials))
    }
}
