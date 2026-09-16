use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Specifies optional parameters for [/institutions/get_by_id]. If provided, must not be null.
///
/// [/instutions/get_by_id]: https://plaid.com/docs/api/institutions/#institutionsget_by_id
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct InstitutionRequestOptions {
    /// When true, return an institution's logo, brand color, and URL. When available, the bank's
    /// logo is returned as a base64 encoded 152x152 PNG, the brand color is in hexadecimal format.
    /// The default value is false.
    ///
    /// Note that Plaid does not own any of the logos shared by the API and that by accessing or
    /// using these logos, you agree that you are doing so at your own risk and will, if necessary,
    /// obtain all required permissions from the appropriate rights holders and adhere to any
    /// applicable usage guidelines. Plaid disclaims all express or implied warranties with respect
    /// to the logos.
    pub include_optional_metadata: bool,
    /// If true, the response will include status information about the institution.
    ///
    /// Note that institution status is not available in the Sandbox environment.
    pub include_status: bool,
    /// When true, returns metadata related to the Auth product indicating which auth methods are
    /// supported.
    pub include_auth_metadata: bool,
    /// When true, returns metadata related to the Payment Initiation product indicating which
    /// payment configurations are supported.
    pub include_payment_initiation_metadata: bool,
}

/// The response for perforimng an `institution` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstitutionResponse {
    /// Details relating to a specific financial institution
    pub institution: Institution,
    /// A unique identifier for the request, which can be used for troubleshooting. This identifier,
    /// like all Plaid identifiers, is case sensitive.
    pub request_id: String,
}

/// Details relating to a specific financial institution
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Institution {
    /// Unique identifier for the institution
    pub institution_id: String,
    /// The official name of the institution
    pub name: String,
    /// A list of the Plaid products supported by the institution. Note that only institutions that
    /// support Instant Auth will return auth in the product array; institutions that do not list
    /// auth may still support other Auth methods such as Instant Match or Automated Micro-deposit
    /// Verification. To identify institutions that support those methods, use the auth_metadata
    /// object. For more details, see Full Auth coverage.
    pub products: Vec<super::SupportedProduct>,
    /// A list of the country codes supported by the institution.
    pub country_codes: Vec<super::SupportedCountry>,
    /// The URL for the institution's website
    pub url: Option<String>,
    /// Hexadecimal representation of the primary color used by the institution
    pub primary_color: Option<String>,
    /// Base64 encoded representation of the institution's logo
    pub logo: Option<String>,
    /// A list of routing numbers known to be associated with the institution. This list is
    /// provided for the purpose of looking up institutions by routing number. It is not
    /// comprehensive and should never be used as a complete list of routing numbers for an
    /// institution.
    pub routing_numbers: Vec<String>,
    /// A partial list of DTC numbers associated with the institution.
    #[serde(default)]
    pub dtc_numbers: Vec<String>,
    /// Indicates that the institution has an OAuth login flow.
    pub oauth: bool,
    /// The status of an institution is determined by the health of its Item logins, Transactions
    /// updates, Investments updates, Liabilities updates, Auth requests, Balance requests, Identity
    /// requests, Investments requests, and Liabilities requests. A login attempt is conducted
    /// during the initial Item add in Link. If there is not enough traffic to accurately calculate
    /// an institution's status, Plaid will return null rather than potentially inaccurate data.
    ///
    /// Institution status is accessible in the Dashboard and via the API using the
    /// [/institutions/get_by_id] endpoint with the include_status option set to true. Note that
    /// institution status is not available in the Sandbox environment.
    ///
    /// [/instutions/get_by_id]: https://plaid.com/docs/api/institutions/#institutionsget_by_id
    pub status: Option<InstitutionStatus>,
    /// Metadata that captures what specific payment configurations an institution supports when
    /// making Payment Initiation requests.
    pub payment_initiation_metadata: Option<PaymentInitiationMetadata>,
    /// Metadata that captures information about the Auth features of an institution.
    pub auth_metadata: Option<AuthMetadata>,
}

/// The status of an institution is determined by the health of its Item logins, Transactions
/// updates, Investments updates, Liabilities updates, Auth requests, Balance requests, Identity
/// requests, Investments requests, and Liabilities requests. A login attempt is conducted during
/// the initial Item add in Link. If there is not enough traffic to accurately calculate an
/// institution's status, Plaid will return null rather than potentially inaccurate data.
///
/// Institution status is accessible in the Dashboard and via the API using the
/// [/institutions/get_by_id] endpoint with the include_status option set to true. Note that
/// institution status is not available in the Sandbox environment.
///
/// [/instutions/get_by_id]: https://plaid.com/docs/api/institutions/#institutionsget_by_id
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct InstitutionStatus {
    /// The status of Item login attempts for the institution.
    #[serde(default)]
    pub item_logins: Option<RequestStatus>,
    /// The status of Transactions updates for the institution.
    #[serde(default)]
    pub transactions_updates: Option<RequestStatus>,
    /// The status of Auth requests for the institution.
    #[serde(default)]
    pub auth: Option<RequestStatus>,
    /// The status of Identity requests for the institution.
    #[serde(default)]
    pub identity: Option<RequestStatus>,
    /// The status of Investments updates for the institution.
    #[serde(default)]
    pub investments_updates: Option<RequestStatus>,
    /// The status of Liabilities updates for the institution.
    #[serde(default)]
    pub liabilities_updates: Option<RequestStatus>,
    /// The status of Liabilities requests for the institution.
    #[serde(default)]
    pub liabilities: Option<RequestStatus>,
    /// The status of Investments requests for the institution.
    #[serde(default)]
    pub investments: Option<RequestStatus>,
    /// Details of recent health incidents associated with the institution.
    #[serde(default)]
    pub health_incidents: Option<Vec<HealthIncident>>,
}

/// A representation of the status health of a request type. Auth requests, Balance requests,
/// Identity requests, Investments requests, Liabilities requests, Transactions updates,
/// Investments updates, Liabilities updates, and Item logins each have their own status object.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequestStatus {
    /// This field is deprecated in favor of the breakdown object, which provides more granular
    /// institution health data.
    ///
    /// HEALTHY: the majority of requests are successful
    ///
    /// DEGRADED: only some requests are successful
    ///
    /// DOWN: all requests are failing
    #[deprecated = "This field is deprecated in favor of the breakdown object, which provides more \
    granular institution health data."]
    pub status: String,
    /// ISO 8601 formatted timestamp of the last status change for the institution.
    pub last_status_change: DateTime<Utc>,
    /// A detailed breakdown of the institution's performance for a request type. The values for
    /// success, error_plaid, and error_institution sum to 1.
    pub breakdown: Breakdown,
}

/// A detailed breakdown of the institution's performance for a request type. The values for
/// success, error_plaid, and error_institution sum to 1.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Breakdown {
    /// The percentage of login attempts that are successful, expressed as a decimal.
    pub success: f64,
    /// The percentage of logins that are failing due to an internal Plaid issue, expressed as a
    /// decimal.
    pub error_plaid: f64,
    /// The percentage of logins that are failing due to an issue in the institution's system,
    /// expressed as a decimal.
    pub error_institution: f64,
    /// How frequently data for subscription products like Investments, Transactions, and
    /// Liabilities is being refreshed for the institution.
    ///
    /// The `refresh_interval` may be `DELAYED` or `STOPPED` even when the success rate is high.
    /// This value is only returned for Transactions, Investments, and Liabilities status
    /// breakdowns.
    ///
    /// Possible values: `NORMAL`, `DELAYED`, `STOPPED`
    #[serde(default)]
    pub refresh_interval: Option<String>,
}

/// Details of recent health incidents associated with the institution.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HealthIncident {
    /// The start date of the incident, in ISO 8601 format, e.g. "2020-10-30T15:26:48Z"
    pub start_date: DateTime<Utc>,
    /// The end date of the incident, in ISO 8601 format, e.g. "2020-10-30T15:26:48Z".
    ///
    /// `None` while the incident is ongoing.
    #[serde(default)]
    pub end_date: Option<DateTime<Utc>>,
    /// The title of the incident
    pub title: String,
    /// Updates on the health incident.
    pub incident_updates: Vec<IncidentUpdate>,
}

/// Updates on the health incident.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IncidentUpdate {
    /// The content of the update.
    #[serde(default)]
    pub description: Option<String>,
    /// The status of the incident.
    ///
    /// Possible values: INVESTIGATING, IDENTIFIED, SCHEDULED, RESOLVED, UNKNOWN
    #[serde(default)]
    pub status: Option<String>,
    /// The date when the update was published, in ISO 8601 format, e.g. "2020-10-30T15:26:48Z".
    #[serde(default)]
    pub updated_date: Option<DateTime<Utc>>,
}

/// Metadata that captures what specific payment configurations an institution supports when
/// making Payment Initiation requests.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaymentInitiationMetadata {
    /// Indicates whether the institution supports payments from a different country.
    pub supports_international_payments: bool,
    /// Indicates whether the institution supports SEPA Instant payments.
    pub supports_sepa_instant: bool,
    /// A mapping of currency to maximum payment amount (denominated in the smallest unit of
    /// currency) supported by the institution.
    ///
    /// Example: {"GBP": "10000"}
    pub maximum_payment_amount: HashMap<String, String>,
    /// Indicates whether the institution supports returning refund details when initiating a payment.
    pub supports_refund_details: bool,
    /// Indicates whether the institution supports payment consents.
    #[serde(default)]
    pub supports_payment_consents: bool,
    /// Metadata specifically related to valid Payment Initiation standing order configurations for
    /// the institution.
    pub standing_order_metadata: Option<StandingOrderMetadata>,
}

/// Metadata specifically related to valid Payment Initiation standing order configurations for the institution.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StandingOrderMetadata {
    /// Indicates whether the institution supports closed-ended standing orders by providing
    /// an end date.
    pub supports_standing_order_end_date: bool,
    /// This is only applicable to MONTHLY standing orders. Indicates whether the institution
    /// supports negative integers (-1 to -5) for setting up a MONTHLY standing order relative
    /// to the end of the month.
    pub supports_standing_order_negative_execution_days: bool,
    /// A list of the valid standing order intervals supported by the institution.
    ///
    /// Possible values: WEEKLY, MONTHLY
    ///
    /// Min length: 1
    pub valid_standing_order_intervals: Vec<String>,
}

/// Metadata that captures information about the Auth features of an institution.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthMetadata {
    /// Metadata specifically related to which auth methods an institution supports.
    pub supported_methods: Option<SupportedMethods>,
}

/// Metadata specifically related to which auth methods an institution supports.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SupportedMethods {
    /// Indicates if instant auth is supported.
    pub instant_auth: bool,
    /// Indicates if instant match is supported.
    pub instant_match: bool,
    /// Indicates if automated micro-deposits are supported.
    pub automated_micro_deposits: bool,
    /// Indicates if instant micro-deposits are supported.
    #[serde(default)]
    pub instant_micro_deposits: bool,
}
