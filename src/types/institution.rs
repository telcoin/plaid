use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Specifies optional parameters for [/institutions/get_by_id]. If provided, must not be null.
///
/// [/instutions/get_by_id]: https://plaid.com/docs/api/institutions/#institutionsget_by_id
#[derive(Serialize, Deserialize, Clone, Debug)]
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
    ///
    /// Possible values: assets, auth, balance, identity, investments, liabilities,
    /// payment_initiation, identity_verification, transactions, credit_details, income,
    /// income_verification, deposit_switch, standing_orders, transfer, employment,
    /// recurring_transactions
    pub products: Vec<String>,
    /// A list of the country codes supported by the institution.
    ///
    /// Possible values: US, GB, ES, NL, FR, IE, CA, DE, IT
    pub country_codes: Vec<String>,
    /// The URL for the institution's website
    pub url: Option<String>,
    /// Hexadecimal representation of the primary color used by the institution
    pub primary_color: Option<String>,
    /// Base64 encoded representation of the institution's logo
    pub logo: Option<String>,
    /// A partial list of routing numbers associated with the institution. This list is provided
    /// for the purpose of looking up institutions by routing number. It is not comprehensive and
    /// should never be used as a complete list of routing numbers for an institution.
    pub routing_numbers: Vec<String>,
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstitutionStatus {
    /// A representation of the status health of a request type. Auth requests, Balance requests,
    /// Identity requests, Investments requests, Liabilities requests, Transactions updates,
    /// Investments updates, Liabilities updates, and Item logins each have their own status object.
    pub item_logins: RequestStatus,
    /// A representation of the status health of a request type. Auth requests, Balance requests,
    /// Identity requests, Investments requests, Liabilities requests, Transactions updates,
    /// Investments updates, Liabilities updates, and Item logins each have their own status object.
    pub transactions_updates: RequestStatus,
    /// A representation of the status health of a request type. Auth requests, Balance requests,
    /// Identity requests, Investments requests, Liabilities requests, Transactions updates,
    /// Investments updates, Liabilities updates, and Item logins each have their own status object.
    pub auth: RequestStatus,
    /// A representation of the status health of a request type. Auth requests, Balance requests,
    /// Identity requests, Investments requests, Liabilities requests, Transactions updates,
    /// Investments updates, Liabilities updates, and Item logins each have their own status object.
    pub identity: RequestStatus,
    /// A representation of the status health of a request type. Auth requests, Balance requests,
    /// Identity requests, Investments requests, Liabilities requests, Transactions updates,
    /// Investments updates, Liabilities updates, and Item logins each have their own status object.
    pub investment_update: RequestStatus,
    /// A representation of the status health of a request type. Auth requests, Balance requests,
    /// Identity requests, Investments requests, Liabilities requests, Transactions updates,
    /// Investments updates, Liabilities updates, and Item logins each have their own status object.
    pub liabilities_updates: RequestStatus,
    /// A representation of the status health of a request type. Auth requests, Balance requests,
    /// Identity requests, Investments requests, Liabilities requests, Transactions updates,
    /// Investments updates, Liabilities updates, and Item logins each have their own status object.
    pub liabilities: RequestStatus,
    /// A representation of the status health of a request type. Auth requests, Balance requests,
    /// Identity requests, Investments requests, Liabilities requests, Transactions updates,
    /// Investments updates, Liabilities updates, and Item logins each have their own status object.
    pub investments: RequestStatus,
    /// Details of recent health incidents associated with the institution.
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
    /// The refresh_interval may be DELAYED or STOPPED even when the success rate is high. This
    /// value is only returned for Transactions status breakdowns.
    pub refresh_interval: String,
}

/// Details of recent health incidents associated with the institution.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HealthIncident {
    /// The start date of the incident, in ISO 8601 format, e.g. "2020-10-30T15:26:48Z"
    pub start_date: DateTime<Utc>,
    /// The end date of the incident, in ISO 8601 format, e.g. "2020-10-30T15:26:48Z".
    pub end_date: DateTime<Utc>,
    /// The title of the incident
    pub title: String,
    /// Updates on the health incident.
    pub incident_updates: Vec<IncidentUpdate>,
}

/// Updates on the health incident.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IncidentUpdate {
    /// The content of the update.
    pub description: String,
    /// The status of the incident.
    ///
    /// Possible values: INVESTIGATING, IDENTIFIED, SCHEDULED, RESOLVED, UNKNOWN
    pub status: String,
    /// The date when the update was published, in ISO 8601 format, e.g. "2020-10-30T15:26:48Z".
    pub updated_date: DateTime<Utc>,
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
    /// Indicates if automated microdeposits are supported.
    pub automated_micro_deposits: bool,
}
