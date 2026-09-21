# plaid

[![CI](https://github.com/telcoin/plaid/workflows/CI/badge.svg)](https://github.com/telcoin/plaid/actions?query=workflow%3ACI)

An unofficial Rust client library for the [Plaid API].

Types track the [`2020-09-14`] version of the Plaid API.

[`2020-09-14`]: https://plaid.com/docs/api/versioning/

### Example

1. Add the following to your `Cargo.toml`:

   ```toml
   [dependencies]
   plaid = { git = "https://github.com/telcoin/plaid.git", tag = "v0.12.0" }
   tokio = { version = "1", features = ["full"] }
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
        let public_token = client
            .sandbox_create_public_token(&plaid::SandboxCreatePublicTokenRequest::default())
            .await?
            .public_token;

        let access_token = client
            .exchange_public_token(&public_token)
            .await?
            .access_token;

        let _accounts = client.accounts(&access_token).await?.accounts;

        Ok(())
    }
   ```

### Verifying webhooks

Plaid signs every webhook it sends, and putting an unsigned endpoint on the
Internet means any caller can make your service store — or act on — a delivery
Plaid never sent. Checking the signature needs a crypto stack, so it lives
behind a feature:

```toml
[dependencies]
plaid = { git = "https://github.com/telcoin/plaid.git", tag = "v0.12.0", features = ["webhook-verification"] }
```

```rust
let verifier = plaid::WebhookVerifier::new(client);

// `header` is the request's `Plaid-Verification` value, and `body` the exact
// bytes that arrived.
match verifier.verify(header, body).await {
    Ok(verified) => {
        let webhook: plaid::Webhook = serde_json::from_slice(body)?;
        // `verified.fingerprint` identifies this delivery, for deduplicating
        // redeliveries of the same event.
    }
    // The key could not be fetched, so nothing was decided: answer in a way
    // that asks Plaid to send the delivery again.
    Err(error) if error.is_inconclusive() => return Err(error.into()),
    // The delivery is not Plaid's. Drop it.
    Err(_) => {}
}
```

A signature proves a delivery came from Plaid, not that it is still true, so a
handler acting on something consequential should still confirm the current state
with Plaid. What verification protects is the step before that: nothing
unverified needs to be written down.

[plaid api]: https://plaid.com/
