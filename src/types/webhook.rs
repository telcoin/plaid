//! Webhooks

use serde::{Deserialize, Serialize};

/// A broad categorization of the error. Safe for programmatic use.
#[derive(Serialize, Deserialize, Debug)]
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
    /// Asset Report Error
    AssetReportError,
    /// Recaptcha Error
    RecaptchaError,
    /// OAuth Error
    OauthError,
    /// Payment Error
    PaymentError,
    /// Bank Transfer Error
    BankTransferError,
    /// Income Verification Error
    IncomeVerificationError,
}

/// We use standard HTTP response codes for success and failure notifications, and our errors are
/// further classified by error_type. In general, 200 HTTP codes correspond to success, 40X codes
/// are for developer- or user-related failures, and 50X codes are for Plaid-related issues.
/// Error fields will be null if no error has occurred.
#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookError {
    /// A user-friendly representation of the error code. `null` if the error is not related to user
    /// action. This may change over time and is not safe for programmatic use.
    pub display_message: Option<String>,
    /// The particular error code
    pub error_code: String,
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
    pub causes: Option<Vec<String>>,
    /// The HTTP status code associated with the error. This will only be returned in the response
    /// body when the error information is provided via a webhook.
    pub status: i32,
    /// The URL of a Plaid documentation page with more information about the error
    pub documentation_url: Option<String>,
    /// Suggested steps for resolving the error
    pub suggested_action: Option<String>,
}

/// The type of webhook
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
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
