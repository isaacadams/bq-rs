# Usage

There are three ways to load authentication credentials for google services.

```rust
// 1. load credentials from an environment variable
let authentication = gauthenticator::from_environment_variable("SOME_ENV_VAR");
// 2. load credentials from a file path
let authentication = gauthenticator::from_file("/path/to/credentials.json");
// 3. load credentials from your machine's environment using well known locations
let authentication = gauthenticator::from_env().authentication();

let Some(authentication) = authentication else {
    panic!("failed to find credentials");
};

// log out the authentication details
log::debug!("{}", authentication.message());

// load project id from user input or from the service account file
let project_id = authentication.project_id().expect("project id is required");

// create the bearer token
let token = authentication.token(None)?;
```
