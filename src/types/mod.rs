//! Request and response types.

use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

pub use account::*;
pub use auth::*;
pub use institution::*;
pub use item::*;
pub use secrecy::{ExposeSecret, SecretString};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
pub use token::*;

// Declared first so `wire_enum!` is in scope for the modules below.
#[macro_use]
pub(crate) mod serde_utils;

mod account;
mod auth;
mod institution;
mod item;
mod token;
pub mod webhook;

/// A [secure] representation of a [Plaid API secret].
///
/// [secure]: https://docs.rs/secrecy/
/// [Plaid API secret]: https://plaid.com/docs/quickstart/glossary/#secret
#[derive(Clone, Debug)]
pub struct Secret(SecretString);

impl Secret {
    fn new(secret: String) -> Self {
        Self(SecretString::from(secret))
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

/// API environments to differentiate between the testing environment
/// (`Sandbox`) and live, billed, unrestricted API access (`Production`), plus
/// a [`Custom`] escape hatch for pointing the client at an arbitrary base URL.
///
/// See [API Host](https://plaid.com/docs/api/#api-host).
///
/// [`Custom`]: Environment::Custom
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Environment {
    /// Sandbox environment (`https://sandbox.plaid.com`).
    ///
    /// Suitable for developer testing.
    Sandbox,

    /// Development environment (`https://development.plaid.com`).
    ///
    /// *Deprecated*: Plaid has retired the Development environment; it is no
    /// longer listed as an API host and requests against it will not succeed.
    /// Use [`Environment::Sandbox`] for testing and
    /// [`Environment::Production`] for live traffic.
    #[deprecated = "Plaid has retired the Development environment; use `Sandbox` or `Production`"]
    Development,

    /// Live production environment (`https://production.plaid.com`).
    ///
    /// All activity in the Production environment will be billed. When you’re
    /// getting ready to launch into Production, please request Production API
    /// access via the Dashboard.
    Production,

    /// An arbitrary base URL, such as a mock server or a proxy.
    ///
    /// Endpoint paths are appended to this value, so it should be a scheme and
    /// host with no trailing path segment — for example
    /// `http://localhost:8080`, against which `/accounts/get` is requested as
    /// `http://localhost:8080/accounts/get`. A trailing `/` is ignored.
    ///
    /// ```
    /// let client = plaid::Client::new(
    ///     "test_client_id",
    ///     "test_secret".to_string(),
    ///     plaid::Environment::Custom("http://localhost:8080".to_string()),
    /// );
    /// ```
    Custom(String),
}

impl Environment {
    /// The base URL that requests are sent to, without a trailing `/`.
    pub fn base_url(&self) -> &str {
        #[allow(deprecated)]
        match self {
            Environment::Sandbox => "https://sandbox.plaid.com",
            Environment::Development => "https://development.plaid.com",
            Environment::Production => "https://production.plaid.com",
            Environment::Custom(url) => url.trim_end_matches('/'),
        }
    }
}

impl FromStr for Environment {
    // TODO: make an `Error` type.
    type Err = String;

    /// Parses one of the named environments, or an `http://` or `https://` URL
    /// as [`Environment::Custom`].
    ///
    /// A scheme is required for a custom URL: without one, a typo such as
    /// `sandbx` would otherwise be silently accepted as a base URL.
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowercased = s.to_ascii_lowercase();
        match lowercased.as_str() {
            "production" => Ok(Environment::Production),
            #[allow(deprecated)]
            "development" => Ok(Environment::Development),
            "sandbox" => Ok(Environment::Sandbox),
            url if url.starts_with("http://") || url.starts_with("https://") => {
                Ok(Environment::Custom(s.to_string()))
            }
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
    /// Writes the name of a named environment, or the base URL of a
    /// [`Environment::Custom`] one, so that the output parses back to an equal
    /// value via [`FromStr`].
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[allow(deprecated)]
        let env = match self {
            Environment::Production => "production",
            Environment::Development => "development",
            Environment::Sandbox => "sandbox",
            Environment::Custom(_) => self.base_url(),
        };
        f.write_str(env)
    }
}

/// Metadata about a requested `Item`.
///
/// See [Item schema](https://plaid.com/docs/api/items/#item-get-response-item).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    /// The Plaid Item ID. The `item_id` is always unique; linking the same
    /// account at the same institution twice will result in two Items with
    /// different `item_id` values. Like all Plaid identifiers, the `item_id` is
    /// case-sensitive.
    pub item_id: String,

    /// The Plaid Institution ID associated with the Item. Field is `null` for
    /// Items created without an institution connection, such as Items created
    /// via Same Day Micro-deposits.
    pub institution_id: Option<String>,

    /// The name of the institution associated with the Item. Field is `null`
    /// for Items created without an institution connection, such as Items
    /// created via Same Day Micro-deposits.
    #[serde(default)]
    pub institution_name: Option<String>,

    // TODO: sometimes this is an empty string instead of `None`
    /// The URL registered to receive webhooks for the Item.
    pub webhook: Option<String>,

    /// The method used to populate Auth data for the Item.
    ///
    /// This field is only populated for Items that have had Auth numbers data
    /// set on at least one of their accounts, and will be `null` otherwise. For
    /// info about the various flows, see Plaid's [Auth coverage
    /// documentation](https://plaid.com/docs/auth/coverage/).
    #[serde(default)]
    pub auth_method: Option<AuthMethod>,

    /// We use standard HTTP response codes for success and failure
    /// notifications, and our errors are further classified by error_type. In
    /// general, 200 HTTP codes correspond to success, 40X codes are for
    /// developer- or user-related failures, and 50X codes are for Plaid-related
    /// issues. Error fields will be null if no error has occurred.
    pub error: Option<crate::ApiError>,

    /// A list of products available for the Item that have not yet been
    /// accessed.
    ///
    /// The contents of this array will be mutually exclusive with
    /// `billed_products`.
    pub available_products: Option<Vec<SupportedProduct>>,

    /// A list of products that have been billed for the Item.
    ///
    /// *Note*: `billed_products` is populated in all environments but only
    /// requests in Production are billed.
    pub billed_products: Option<Vec<SupportedProduct>>,

    /// A list of products added to the Item.
    ///
    /// In almost all cases, this will be the same as the `billed_products`
    /// field. For some products, it is possible for the product to be added to
    /// an Item but not yet billed (e.g. Assets, before `/asset_report/create`
    /// has been called, or Auth or Identity when added as Optional Products but
    /// before their endpoints have been called).
    #[serde(default)]
    pub products: Option<Vec<SupportedProduct>>,

    /// A list of products that the user has consented to for the Item via [Data
    /// Transparency Messaging].
    ///
    /// This will consist of all products where both of the following are true:
    /// the user has consented to the required data scopes for that product and
    /// you have Production access for that product.
    ///
    /// [Data Transparency Messaging]: https://plaid.com/docs/link/data-transparency-messaging-migration-guide
    #[serde(default)]
    pub consented_products: Option<Vec<SupportedProduct>>,

    /// The [RFC 3339] timestamp after which the consent provided by the end
    /// user will expire. Upon consent expiration, the item will enter the
    /// `ITEM_LOGIN_REQUIRED` error state. To circumvent the
    /// `ITEM_LOGIN_REQUIRED` error and maintain continuous consent, the end
    /// user can reauthenticate via Link’s update mode in advance of the consent
    /// expiration time.
    ///
    /// *Note*: This is only relevant for certain OAuth-based institutions. For
    /// all other institutions, this field will be `null`.
    ///
    /// [RFC 3339]: https://tools.ietf.org/html/rfc3339
    pub consent_expiration_time: Option<chrono::DateTime<chrono::FixedOffset>>,

    /// Indicates whether an Item requires user interaction to be updated, which
    /// can be the case for Items with some forms of two-factor authentication.
    #[serde(default)]
    pub update_type: Option<UpdateType>,
}

/// Indicates whether an `Item` requires user interaction to be updated.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateType {
    /// Item can be updated in the background.
    Background,

    /// Item requires user interaction to be updated.
    UserPresentRequired,
}

/// The method used to populate Auth data for an `Item`.
///
/// See [Auth coverage](https://plaid.com/docs/auth/coverage/).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthMethod {
    /// The Item's Auth data was provided directly by the user's institution
    /// connection.
    InstantAuth,

    /// The Item's Auth data was provided via the Instant Match fallback flow.
    InstantMatch,

    /// The Item's Auth data was provided via the Automated Micro-deposits flow.
    AutomatedMicrodeposits,

    /// The Item's Auth data was provided via the Same-Day Micro-deposits flow.
    SameDayMicrodeposits,

    /// The Item's Auth data was provided via the Instant Micro-deposits flow.
    InstantMicrodeposits,

    /// The Item's Auth data was provided via the Database Match flow.
    DatabaseMatch,

    /// The Item's Auth data was provided via the Database Insights flow.
    DatabaseInsights,

    /// The Item's Auth data was provided via `/transfer/migrate_account`.
    TransferMigrated,

    /// The Item's Auth data for Investments Move was provided via a fallback
    /// flow.
    InvestmentsFallback,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_environments_map_to_plaid_hosts() {
        assert_eq!(Environment::Sandbox.base_url(), "https://sandbox.plaid.com");
        assert_eq!(
            Environment::Production.base_url(),
            "https://production.plaid.com"
        );
    }

    #[test]
    fn custom_environment_uses_the_given_url() {
        let custom = Environment::Custom("http://localhost:8080".to_string());
        assert_eq!(custom.base_url(), "http://localhost:8080");
    }

    #[test]
    fn custom_environment_ignores_a_trailing_slash() {
        // otherwise endpoint paths would produce `http://localhost:8080//auth/get`
        let custom = Environment::Custom("http://localhost:8080/".to_string());
        assert_eq!(custom.base_url(), "http://localhost:8080");
    }

    #[test]
    fn urls_parse_as_custom_environments() {
        assert_eq!(
            "http://localhost:8080".parse::<Environment>().unwrap(),
            Environment::Custom("http://localhost:8080".to_string()),
        );
        assert_eq!(
            "https://mock.example.com".parse::<Environment>().unwrap(),
            Environment::Custom("https://mock.example.com".to_string()),
        );
    }

    #[test]
    fn parsing_a_url_preserves_its_case() {
        let environment = "https://Mock.Example.com/Plaid"
            .parse::<Environment>()
            .unwrap();
        assert_eq!(environment.base_url(), "https://Mock.Example.com/Plaid");
    }

    #[test]
    fn named_environments_parse_case_insensitively() {
        assert_eq!(
            "SANDBOX".parse::<Environment>().unwrap(),
            Environment::Sandbox
        );
        assert_eq!(
            "Production".parse::<Environment>().unwrap(),
            Environment::Production
        );
    }

    #[test]
    fn a_typo_is_an_error_rather_than_a_base_url() {
        assert!("sandbx".parse::<Environment>().is_err());
        assert!("localhost:8080".parse::<Environment>().is_err());
    }

    #[test]
    fn display_round_trips_through_from_str() {
        for environment in [
            Environment::Sandbox,
            Environment::Production,
            Environment::Custom("http://localhost:8080".to_string()),
        ] {
            let displayed = environment.to_string();
            assert_eq!(
                displayed.parse::<Environment>().unwrap(),
                environment,
                "`{}` did not round-trip",
                displayed,
            );
        }
    }
}
