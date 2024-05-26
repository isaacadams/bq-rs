mod credentials;
mod sign;
mod token;

pub use credentials::*;
pub use token::*;

pub fn auto_load_service_account_key() -> CredentialFileResult {
    CredentialsFile::from_well_known_env().or(CredentialsFile::from_well_known_file())
}
