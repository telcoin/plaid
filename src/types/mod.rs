//! Request and response types.

use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

pub use account::*;
pub use auth::*;
use secrecy::{ExposeSecret, SecretString};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
pub use token::*;

mod account;
mod auth;
pub(crate) mod serde_utils;
mod token;

/// A [secure] representation of a [Plaid API secret].
///
/// [secure]: https://docs.rs/secrecy/
/// [Plaid API secret]: https://plaid.com/docs/quickstart/glossary/#secret
#[derive(Clone, Debug)]
pub struct Secret(SecretString);

impl Secret {
    fn new(secret: String) -> Self {
        Self(SecretString::new(secret))
    }
}

impl From<SecretString> for Secret {
    fn from(value: SecretString) -> Self {
        Secret(value)
    }
}

impl From<String> for Secret {
    fn from(value: String) -> Self {
        Secret::new(value)
    }
}

impl Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.expose_secret().serialize(serializer)
    }
}

/// API environments to differentiate between testing environments (`Sandbox`
/// and `Development`) and live, billed, unrestricted API access (`Production`).
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Environment {
    /// Sandbox environment.
    ///
    /// Suitable for developer testing.
    Sandbox,

    /// Development environment.
    ///
    /// The `Development` environment supports up to 100 live `Items`.
    Development,

    /// Live production environment.
    ///
    /// All activity in the Production environment will be billed. When you’re
    /// getting ready to launch into Production, please request Production API
    /// access via the Dashboard.
    Production,
}

impl FromStr for Environment {
    // TODO: make an `Error` type.
    type Err = String;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "production" => Ok(Environment::Production),
            "development" => Ok(Environment::Development),
            "sandbox" => Ok(Environment::Sandbox),
            val => Err(format!("invalid Plaid Environment: `{}`", val)),
        }
    }
}

impl TryFrom<String> for Environment {
    // TODO: make an `Error` type.
    type Error = String;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl<'a> TryFrom<&'a str> for Environment {
    // TODO: make an `Error` type.
    type Error = String;

    #[inline]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl fmt::Display for Environment {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let env = match &self {
            Environment::Production => "production",
            Environment::Development => "development",
            Environment::Sandbox => "sandbox",
        };
        write!(f, "{}", env)
    }
}

/// Metadata about a requested `Item`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    /// The Plaid Item ID. The `item_id` is always unique; linking the same
    /// account at the same institution twice will result in two Items with
    /// different `item_id` values. Like all Plaid identifiers, the `item_id` is
    /// case-sensitive.
    pub item_id: String,

    /// The Plaid Institution ID associated with the Item. Field is null for
    /// Items created via Same Day Micro-deposits.
    pub institution_id: Option<String>,

    // TODO: sometimes this is an empty string instead of `None`
    /// The URL registered to receive webhooks for the Item.
    pub webhook: Option<String>,

    /// We use standard HTTP response codes for success and failure
    /// notifications, and our errors are further classified by error_type. In
    /// general, 200 HTTP codes correspond to success, 40X codes are for
    /// developer- or user-related failures, and 50X codes are for Plaid-related
    /// issues. Error fields will be null if no error has occurred.
    pub error: Option<serde_json::Value>,

    // TODO: make a `Product` enum
    /// A list of products available for the Item that have not yet been
    /// accessed.
    pub available_products: Option<Vec<String>>,

    /// A list of products that have been billed for the Item.
    ///
    /// *Note*: billed_products is populated in all environments but only
    /// requests in Production are billed.
    pub billed_products: Option<Vec<String>>,

    /// The [RFC 3339] timestamp after which the consent provided by the end
    /// user will expire. Upon consent expiration, the item will enter the
    /// `ITEM_LOGIN_REQUIRED` error state. To circumvent the
    /// `ITEM_LOGIN_REQUIRED` error and maintain continuous consent, the end
    /// user can reauthenticate via Link’s update mode in advance of the consent
    /// expiration time.
    ///
    /// *Note*: This is only relevant for European institutions subject to PSD2
    /// regulations mandating a 90-day consent window. For all other
    /// institutions, this field will be `null`.
    ///
    /// [RFC 3339]: https://tools.ietf.org/html/rfc3339
    pub consent_expiration_time: Option<chrono::DateTime<chrono::FixedOffset>>,
}
