# plaid

[![CI](https://github.com/telcoin/plaid/workflows/CI/badge.svg)](https://github.com/telcoin/plaid/actions?query=workflow%3ACI)

An unofficial Rust client library for the [Plaid API].

### Example

1. Add the following to your `Cargo.toml`:

   ```toml
   [dependencies]
   plaid = { git = "https://github.com/telcoin/plaid.git", tag = "v0.1.0" }
   tokio = { version = "0.2", features = ["full"] }
   ```

1. Obtain your API credentials from: https://dashboard.plaid.com/team/keys

1. Get started with this example `main.rs`:

   ```rust
    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // or use `plaid::Client::from_env()?`
        let client = plaid::Client::new(
            "your_client_id",
            "your_client_secret",
            plaid::Environment::Sandbox,
        );

        // TODO: use the Link flow instead; https://plaid.com/docs/link/#link-flow
        let public_token = client.sandbox_create_public_token().await?.public_token;

        let access_token = client
            .exchange_public_token(&public_token)
            .await?
            .access_token;

        let _accounts = client.accounts(&access_token).await?.accounts;

        Ok(())
    }
   ```

[plaid api]: https://plaid.com/
