mod credentials;
mod profile;
mod sign;
mod source;
mod token;

pub use credentials::*;
pub use source::*;
pub use token::*;

pub fn auto_load_service_account_key() -> CredentialFileResult {
    CredentialsFile::from_well_known_env().or(CredentialsFile::from_well_known_file())
}
