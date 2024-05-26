mod credentials;
mod profile;
mod sign;
mod token;

/// exports
pub use credentials::*;
pub use token::*;

use profile::{GoogleCloudConfigurationContext, ProfileSchema};
use std::path::Path;

pub struct Authentication {
    loading_from: String,
    credentials: Result<CredentialsSchema, Error>,
    core_profile: Option<ProfileSchema>,
}

impl Authentication {
    pub fn credentials(self) -> Result<CredentialsSchema, Error> {
        self.credentials
    }

    pub fn project_id(&self) -> Option<&str> {
        self.credentials
            .as_ref()
            .ok()
            .and_then(|c| c.project_id())
            .or(self.core_profile.as_ref().map(|p| p.project.as_str()))
    }

    /* pub fn token(&self, audience: Option<String>) -> Result<String, Error> {
        self.credentials?.token(audience)
    } */

    pub fn message(&self) -> String {
        let mut message = String::new();

        message.push('\n');
        message.push_str(self.status());
        message.push(' ');
        message.push_str(&self.loading_from);
        message.push('\n');

        let info = match &self.credentials {
            Ok(c) => format!(
                "email:\t{}\ntype:\t{}",
                c.email().unwrap_or("N/A"),
                c.kind()
            ),
            Err(e) => e.to_string(),
        };

        message.push_str(&info);
        message
    }

    pub fn status(&self) -> &'static str {
        if self.credentials.is_ok() {
            "✅"
        } else {
            "❌"
        }
    }
}

pub struct FromEnv {
    google_application_credentials: Authentication,
    core_profile: Option<Authentication>,
    application_default: Option<Authentication>,
}

impl FromEnv {
    pub fn print(&self) {
        println!(
            "various well known locations have been checked for valid credentials:\n{}\n{}\n{}",
            &self.google_application_credentials.message(),
            &self
                .core_profile
                .as_ref()
                .map(|c| c.message())
                .unwrap_or("N/A".to_string()),
            &self
                .application_default
                .as_ref()
                .map(|c| c.message())
                .unwrap_or("N/A".to_string()),
        );
    }

    pub fn load(self) -> Option<CredentialsSchema> {
        self.google_application_credentials
            .credentials()
            .ok()
            .or(self.core_profile.and_then(|c| c.credentials().ok()))
            .or(self.application_default.and_then(|c| c.credentials().ok()))
    }
}

/// tries loading credentials from various well known locations
pub fn credentials_from_env() -> FromEnv {
    let mut context = GoogleCloudConfigurationContext::new();
    let env = FromEnv {
        google_application_credentials: credentials_from_environment_variable(
            "GOOGLE_APPLICATION_CREDENTIALS",
        ),
        core_profile: context.as_mut().ok().and_then(|cxt| {
            let profile = cxt.profiles.get("core").ok()?;
            let path = cxt.directory.get_profile_adc(&profile);
            let mut auth = crate::credentials_from_file(path);
            auth.core_profile = Some(profile);
            Some(auth)
        }),
        application_default: context
            .as_ref()
            .ok()
            .map(|cxt| credentials_from_file(cxt.directory.get_application_default_credentials())),
    };
    env
}

pub fn credentials_from_environment_variable<S: AsRef<str>>(variable: S) -> Authentication {
    let result = dotenvy::var(variable.as_ref())
        .map_err(|e| Error::FailedToLoad(format!("{} because {}", variable.as_ref(), e)))
        .and_then(|c| CredentialsSchema::deserialize(&c));

    Authentication {
        core_profile: None,
        credentials: result,
        loading_from: format!("env:{}", variable.as_ref()),
    }
}

pub fn credentials_from_file<P: AsRef<Path>>(path: P) -> Authentication {
    let path = path.as_ref();
    let result = std::fs::read_to_string(path)
        .map_err(|e| Error::FailedToLoad(format!("{} because {}", path.display(), e)))
        .and_then(|c| CredentialsSchema::deserialize(&c));

    Authentication {
        core_profile: None,
        credentials: result,
        loading_from: format!("{}", path.display()),
    }
}
