```rust
let sa = gauthenticator::load("./service-account-key.json".as_ref()).unwrap();
let token = sa.access_token().unwrap();
```
