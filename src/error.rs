use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

use reqwest::Error as ReqwestError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents an error that can occur when making an API request.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Error {
    /// An error that was reported by the Plaid API
    Api(ApiError),

    /// An error that ocurred during transport (using "futures-std" feature)
    TransportStd(ReqwestError),
}

/// The error code Plaid answers with for an `Item` it cannot find, including
/// one that has already been removed.
const ITEM_NOT_FOUND: &str = "ITEM_NOT_FOUND";

impl Error {
    /// Whether this is Plaid's `ITEM_NOT_FOUND`, meaning the Item the request
    /// named does not exist.
    ///
    /// **What makes [`Client::remove_item`] idempotent.** Removing an Item that
    /// is already gone answers with this rather than succeeding, so a caller
    /// retrying after a failure part-way through can read it as "the state you
    /// asked for holds":
    ///
    /// ```no_run
    /// # async fn unlink(client: &plaid::Client, access_token: &str) -> Result<(), plaid::Error> {
    /// match client.remove_item(access_token).await {
    ///     Ok(_) => Ok(()),
    ///     Err(error) if error.is_item_not_found() => Ok(()),
    ///     Err(error) => Err(error),
    /// }
    /// # }
    /// ```
    ///
    /// **Narrow on purpose.** This is the one code Plaid documents for an Item
    /// that is already gone, and it is the outcome the caller wanted. Every
    /// other failure — including any other Item error — is a real one, because
    /// the two ways to be wrong here are not symmetrical: reporting a link
    /// revoked when it was not leaves a live credential behind, while refusing
    /// one that really was gone costs a retry that succeeds.
    ///
    /// [`Client::remove_item`]: crate::Client::remove_item
    pub fn is_item_not_found(&self) -> bool {
        matches!(
            self,
            Error::Api(api)
                if api.error_type == ErrorType::ItemError && api.error_code == ITEM_NOT_FOUND
        )
    }
}

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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiError {
    /// A broad categorization of the error. Safe for programatic use.
    pub error_type: ErrorType,

    /// The particular error code. Safe for programmatic use.
    pub error_code: String,

    /// The specific reason for the error code.
    ///
    /// Currently, reasons are only supported for OAuth-based Item errors, and
    /// `null` will be returned otherwise. Safe for programmatic use.
    #[serde(default)]
    pub error_code_reason: Option<String>,

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

    /// The HTTP status code associated with the error.
    ///
    /// This will only be returned in the response body when the client library
    /// is configured to use "errors in the response body" mode.
    #[serde(default)]
    pub status: Option<i64>,

    /// In products where a request can pertain to more than one Item, an error
    /// returned for such a request may have a `causes` array describing which
    /// Items caused the error.
    ///
    /// If there are no errors, `causes` will be an empty array. `causes` will
    /// only be provided for the `ASSET_REPORT_ERROR` type.
    #[serde(default)]
    pub causes: Vec<serde_json::Value>,

    /// A list of the account subtypes that were requested via the
    /// `account_filters` parameter of `/link/token/create`.
    ///
    /// Only returned for the `INVALID_ACCOUNT_SUBTYPE` error code.
    #[serde(default)]
    pub required_account_subtypes: Vec<String>,

    /// A list of the account subtypes that were extracted but did not match the
    /// requested subtypes via the `account_filters` parameter of
    /// `/link/token/create`.
    ///
    /// Only returned for the `INVALID_ACCOUNT_SUBTYPE` error code.
    #[serde(default)]
    pub provided_account_subtypes: Vec<String>,
}

/// See [Error Type](https://plaid.com/docs/errors/#error-schema)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorType {
    /// Occur when an Item may be invalid or not supported on Plaid's platform.
    ItemError,

    /// Occur when there are errors for the requested financial institution.
    InstitutionError,

    /// Occur during planned maintenance and in response to API errors.
    ApiError,

    /// Occur for errors related to Assets endpoints.
    AssetsError,

    /// Occur for errors related to Asset Report endpoints.
    AssetReportError,

    /// Occur for errors related to Base Report endpoints.
    BaseReportError,

    /// Occur for errors related to Payment Initiation endpoints.
    PaymentError,

    /// Occur for errors related to Bank Transfers endpoints.
    BankTransferError,

    /// Occur for errors related to Transfer endpoints.
    TransferError,

    /// Occur for errors related to recurring Transfer endpoints.
    TransferRecurringError,

    /// Occur for errors related to Transfer refund endpoints.
    TransferRefundError,

    /// Occur for errors related to Deposit Switch endpoints.
    ///
    /// *Deprecated*: Plaid has retired the Deposit Switch product.
    #[deprecated = "Plaid has retired the Deposit Switch product"]
    DepositSwitchError,

    /// Occur for errors related to Income endpoints.
    IncomeVerificationError,

    /// Occur for errors related to micro-deposit verification.
    MicrodepositsError,

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

    /// Occur for errors originating from one of Plaid's partners.
    PartnerError,

    /// Occur for errors related to Signal endpoints.
    SignalError,

    /// Occur for errors related to Transactions endpoints.
    TransactionsError,

    /// Occur for errors related to a specific transaction.
    TransactionError,

    /// Occur for errors related to Recurring Transactions endpoints.
    RecurringTransactionsError,

    /// Occur for errors related to Statements endpoints.
    StatementsError,

    /// Occur for errors related to Check Report endpoints.
    CheckReportError,

    /// Occur for errors related to Consumer Report (CRA) endpoints.
    ConsumerReportError,

    /// Occur for errors related to Consumer Report monitoring.
    CraMonitoringError,

    /// Occur for errors related to Credit Profile Report endpoints.
    CreditProfileReportError,

    /// Occur for errors related to `/user/*` endpoints.
    UserError,

    /// Occur when an idempotency key is reused with different request
    /// parameters.
    IdempotencyError,

    /// Occur for errors related to Encompass integrations.
    EncompassError,

    /// Occur for errors related to Enrich endpoints.
    EnrichError,

    /// Occur for errors related to fraud insights.
    FraudInsightsError,

    /// Occur for errors related to Freddie Mac integrations.
    FreddieMacError,

    /// Occur for errors related to Hosted Link delivery.
    LinkDeliveryError,

    /// Occur for errors related to Profile endpoints.
    ProfileError,

    /// Unknown or all other errors.
    #[serde(other)]
    Unknown,
}
