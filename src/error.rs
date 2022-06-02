use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

use reqwest::Error as ReqwestError;
use serde::{Deserialize, Serialize};

/// Represents an error that can occur when making an API request.
#[derive(Debug)]
pub enum Error {
    /// An error that was reported by the Plaid API
    Api(ApiError),

    /// An error that ocurred during transport (using "futures-std" feature)
    TransportStd(ReqwestError),
}

// #[derive(Debug)]
// #[cfg(feature = "webhook-verification")]
// pub enum WebhookVerificationError {
//     Jwt(JwtError),
//     OpenSsl,
//     Other(Box<dyn std::error::Error>),
// }

impl From<ReqwestError> for Error {
    fn from(error: ReqwestError) -> Self {
        Error::TransportStd(error)
    }
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

/// See [Error Schema](https://plaid.com/docs/errors/#error-schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// A broad categorization of the error. Safe for programatic use.
    pub error_type: ErrorType,

    /// The particular error code. Safe for programmatic use.
    pub error_code: String,

    /// A developer-friendly representation of the error code. This may change
    /// over time and is not safe for programmatic use.
    pub error_message: String,

    /// A user-friendly representation of the error code. `null` if the error
    /// is not related to user action. This may change over time and is not
    /// safe for programmatic use.
    pub display_message: Option<String>,

    /// A unique ID identifying the request, to be used for troubleshooting
    /// purposes. This field will be omitted in errors provided by webhooks.
    pub request_id: Option<String>,

    /// The URL of a Plaid documentation page with more information about the
    /// error.
    pub documentation_url: Option<String>,

    /// Suggested steps for resolving the error.
    pub suggested_action: Option<String>,
}

/// See [Error Type](https://plaid.com/docs/errors/#Error-error-type)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorType {
    /// Occur when an Item may be invalid or not supported on Plaid's platform.
    ItemError,

    /// Occur when there are errors for the requested financial institution.
    InstitutionError,

    /// Occur during planned maintenance and in response to API errors.
    ApiError,

    /// Occur for errors related to Asset endpoints.
    AssetReportError,

    /// Occur for errors related to Payment Initiation endpoints.
    PaymentError,

    /// Occur for errors related to Bank Transfers endpoints.
    BankTransferError,

    /// Occur for errors related to Deposit Switch endpoints.
    DepositSwitchError,

    /// Occur for errors related to Income endpoints.
    IncomeVerificationError,

    /// Occur when invalid parameters are supplied in the Sandbox environment.
    SandboxError,

    /// Occur when a request is malformed and cannot be processed.
    InvalidRequest,

    /// Occur when all fields are provided, but the values provided are
    /// incorrect in some way.
    InvalidInput,

    /// Occur when a request is valid, but the output would be unusable for any
    /// supported flow.
    InvalidResult,

    /// Occur when an excessive number of requests are made in a short period
    /// of time.
    RateLimitExceeded,

    /// Occur when a Recaptcha challenge has been presented or failed during
    /// the link process.
    RecaptchaError,

    /// Occur when there is an error in OAuth authentication.
    OauthError,

    /// Unknown or all other errors.
    #[serde(other)]
    Unknown,
}
