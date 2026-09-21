//! Webhooks

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The response from performing an `update_webhook` request
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebhookUpdateResponse {
    /// Metadata about the Item.
    pub item: super::Item,
    /// A unique identifier for the request, which can be used for
    /// troubleshooting. This identifier, like all Plaid identifiers, is case
    /// sensitive.
    pub request_id: String,
}

/// A broad categorization of the error. Safe for programmatic use.
///
/// See [Error Schema](https://plaid.com/docs/errors/#error-schema).
#[derive(Serialize, Deserialize, JsonSchema, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookErrorType {
    /// Invalid Request Error
    InvalidRequest,
    /// Invalid Result Error
    InvalidResult,
    /// Invalid Input Error
    InvalidInput,
    /// Institution Error
    InstitutionError,
    /// Rate Limit Error
    RateLimitExceeded,
    /// API Error
    ApiError,
    /// Item Error
    ItemError,
    /// Assets Error
    AssetsError,
    /// Asset Report Error
    AssetReportError,
    /// Base Report Error
    BaseReportError,
    /// Recaptcha Error
    RecaptchaError,
    /// OAuth Error
    OauthError,
    /// Payment Error
    PaymentError,
    /// Bank Transfer Error
    BankTransferError,
    /// Transfer Error
    TransferError,
    /// Income Verification Error
    IncomeVerificationError,
    /// Micro-deposits Error
    MicrodepositsError,
    /// Sandbox Error
    SandboxError,
    /// Partner Error
    PartnerError,
    /// Signal Error
    SignalError,
    /// Transactions Error
    TransactionsError,
    /// Recurring Transactions Error
    RecurringTransactionsError,
    /// Statements Error
    StatementsError,
    /// Check Report Error
    CheckReportError,
    /// Consumer Report Error
    ConsumerReportError,
    /// User Error
    UserError,
    /// Idempotency Error
    IdempotencyError,
    /// Unknown or all other errors.
    #[serde(other)]
    Unknown,
}

/// We use standard HTTP response codes for success and failure notifications, and our errors are
/// further classified by error_type. In general, 200 HTTP codes correspond to success, 40X codes
/// are for developer- or user-related failures, and 50X codes are for Plaid-related issues.
/// Error fields will be null if no error has occurred.
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct WebhookError {
    /// A user-friendly representation of the error code. `null` if the error is not related to user
    /// action. This may change over time and is not safe for programmatic use.
    pub display_message: Option<String>,
    /// The particular error code
    pub error_code: String,
    /// The specific reason for the error code. Currently, reasons are only supported for
    /// OAuth-based Item errors, and `null` will be returned otherwise.
    #[serde(default)]
    pub error_code_reason: Option<String>,
    /// A developer-friendly representation of the error code. This may change over time and is not
    /// safe for programmatic use.
    pub error_message: String,
    /// A broad categorization of the error
    pub error_type: WebhookErrorType,
    /// A unique ID identifying the request, to be used for troubleshooting purposes. This field
    /// will be omitted in errors provided by webhooks.
    pub request_id: Option<String>,
    /// In the `Assets` product, a request can pertain to more than one `Item`. If an error is returned
    /// for such a request, causes will return an array of errors containing a breakdown of these
    /// errors on the individual `Item` level, if any can be identified.
    /// `causes` will only be provided for the error_type `ASSET_REPORT_ERROR`. `causes` will also not be
    /// populated inside an error nested within a warning object.
    #[serde(default)]
    pub causes: Option<Vec<serde_json::Value>>,
    /// The HTTP status code associated with the error. This will only be returned in the response
    /// body when the error information is provided via a webhook.
    #[serde(default)]
    pub status: Option<i32>,
    /// The URL of a Plaid documentation page with more information about the error
    pub documentation_url: Option<String>,
    /// Suggested steps for resolving the error
    pub suggested_action: Option<String>,
}

/// The type of webhook
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(tag = "webhook_type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookType {
    /// Webhook relating to `Item`
    Item {
        /// Content of the Webhook
        #[serde(flatten)]
        content: crate::ItemWebhook,
    },
}

/// Top level webhook struct
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Webhook {
    /// The type of webhook
    #[serde(flatten)]
    pub webhook_type: WebhookType,
    /// We use standard `HTTP` response codes for success and failure notifications, and our errors are
    /// further classified by `error_type`. In general, `200 HTTP` codes correspond to success, `40X` codes
    /// are for developer- or user-related failures, and `50X` codes are for Plaid-related issues.
    /// Error fields will be `null` if no error has occurred.
    pub error: Option<WebhookError>,
}

/// The response from performing a `webhook_verification_key` request.
///
/// See [/webhook_verification_key/get].
///
/// [/webhook_verification_key/get]: https://plaid.com/docs/api/webhooks/webhook-verification/#webhook_verification_keyget
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebhookVerificationKeyResponse {
    /// A JSON Web Key (JWK) that can be used in conjunction with [JWT
    /// libraries](https://jwt.io/#libraries-io) to verify webhooks.
    pub key: WebhookVerificationKey,

    /// A unique identifier for the request, which can be used for
    /// troubleshooting. This identifier, like all Plaid identifiers, is case
    /// sensitive.
    pub request_id: String,
}

/// A JSON Web Key (JWK) for one of Plaid's webhook signing keys.
///
/// See [/webhook_verification_key/get].
///
/// [/webhook_verification_key/get]: https://plaid.com/docs/api/webhooks/webhook-verification/#webhook_verification_keyget
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebhookVerificationKey {
    /// The algorithm this key signs with — `ES256`.
    pub alg: String,

    /// The curve — `P-256`.
    pub crv: String,

    /// The key type — `EC`.
    pub kty: String,

    /// The intended use of the key — `sig`.
    #[serde(rename = "use")]
    pub usage: String,

    /// The ID of the key, which is what a delivery's JWT header names in its
    /// `kid` field.
    pub kid: String,

    /// The x coordinate of the public key, base64url-encoded.
    pub x: String,

    /// The y coordinate of the public key, base64url-encoded.
    pub y: String,

    /// When this key was created, as a Unix timestamp.
    pub created_at: i64,

    /// When this key was rotated out, as a Unix timestamp, or `null` if it is
    /// still current.
    ///
    /// A key Plaid has retired must not verify anything: it is exactly the key
    /// an attacker would want, because a rotation usually means the old one is
    /// no longer trusted, so `WebhookVerifier` refuses one.
    #[serde(default)]
    pub expired_at: Option<i64>,
}
