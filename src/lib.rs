#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::all)]

//! [Plaid](https://plaid.com/docs) API.
//!
//! See official documentation at: [https://plaid.com/docs](https://plaid.com/docs).
#![cfg_attr(
    all(feature = "futures-std", not(feature = "futures-01")),
    doc = r##"
### Examples

```no_run
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = plaid::Client::from_env()?;

    // TODO: obtain a `public_token` from a client using Link;
    // see: https://plaid.com/docs/link/#link-flow
    let public_token = "".to_string();

    let access_token = client
        .exchange_public_token(&public_token)
        .await?
        .access_token;

    let _accounts = client.accounts(&access_token).await?.accounts;

    Ok(())
}
```
"##
)]

pub use self::error::*;
#[cfg(feature = "futures-01")]
pub use self::lib_futures_01::*;
#[cfg(feature = "futures-std")]
pub use self::lib_futures_std::*;
pub use self::types::*;

mod error;
#[cfg(feature = "futures-01")]
mod lib_futures_01;
#[cfg(feature = "futures-std")]
mod lib_futures_std;
mod types;
