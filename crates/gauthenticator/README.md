# Usage

## Load Service Account

There are two ways to instantiate a service account.

1. Point to a file

```rust
let service_account = gauthenticator::ServiceAccountKey::from_file("./path-to/service-account-key.json").unwrap();
```

2. Using the `GOOGLE_APPLICATION_CREDENTIALS` environment variable

```rust
let service_account = gauthenticator::ServiceAccountKey::from_env().unwrap();
```

## Get Access Token

```rust
let service_account = gauthenticator::ServiceAccountKey::from_env().unwrap();
let token = service_account.access_token(None)?;
```
