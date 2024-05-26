# Usage

## Get Access Token

```rust
let credentials: CredentialsFile = gauthenticator::auto_load_service_account_key().unwrap();
let token = credentials.token(None)?;
```

## Various ways to load a Credential File

1.  Load from any file path:

```rust
let credentials = gauthenticator::CredentialsFile::from_file("./path-to/credentials.json").unwrap();
```

2.  Load from `$GOOGLE_APPLICATION_CREDENTIALS`:

```rust
let credentials = gauthenticator::CredentialsFile::from_well_known_env().unwrap();
```

3.  Load from any environment variable:

```rust
let credentials = gauthenticator::CredentialsFile::from_env("SOME_ENVIRONMENT_VARIABLE_NAME").unwrap();
```

4.  Load using known credentials path (only works if you have `gcloud` installed and have successfully ran `gcloud auth login`)

```rust
let credentials = CredentialsFile::from_well_known_file().unwrap();
```

5.  auto load (tries running methods 2 and 4):

```rust
let credentials = gauthenticator::auto_load_service_account_key().unwrap();
```
