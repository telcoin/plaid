use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

// TODO: make a `link` module?

// TODO: clean API to encode invariants of `CreateLinkTokenRequest`
/// The body for the `create_link_token` request.
///
/// See [/link/token/create].
///
/// [/link/token/create]: https://plaid.com/docs/api/link/#linktokencreate
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateLinkTokenRequest {
    /// The name of your application, as it should be displayed in Link.
    ///
    /// Maximum length of 30 characters.
    pub client_name: String,

    /// The language that Link should be displayed in.
    ///
    /// When using a Link customization, the language configured here must match
    /// the setting in the customization, or the customization will not be
    /// applied.
    pub language: SupportedLanguage,

    /// Specify an array of Plaid-supported country codes using the [ISO 3166-1
    /// alpha-2] country code standard.
    ///
    /// Note that if you initialize with a European country code, your users
    /// will see the European consent panel during the Link flow. If Link is
    /// launched with multiple country codes, only products that you are enabled
    /// for in all countries will be used by Link.
    ///
    /// If using a Link customization, make sure the country codes in the
    /// customization match those specified in `country_codes`. If both
    /// `country_codes` and a Link customization are used, the value in
    /// `country_codes` may override the value in the customization.
    ///
    /// [ISO 3166-1 alpha-2]: https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
    pub country_codes: Vec<SupportedCountry>,

    /// An object specifying information about the end user who will be linking
    /// their account.
    pub user: EndUser,

    // TODO: should this be ser/de with `serde_utils::default_on_null`?
    /// List of Plaid product(s) the linked Item must support. If launching Link
    /// in update mode, should be omitted; required otherwise.
    ///
    /// In Production, you will be billed for each product that you specify when
    /// initializing Link. Note that a product cannot be removed from an Item
    /// once the Item has been initialized with that product. To stop billing on
    /// an Item for subscription-based products, such as Liabilities,
    /// Investments, and Transactions, remove the Item via `/item/remove`.
    #[serde(default, with = "super::serde_utils::default_on_null")]
    pub products: Vec<SupportedProduct>,

    /// List of Plaid product(s) to use only if the institution and account(s)
    /// selected by the user support the product.
    ///
    /// The products will be added to the Item, but not billed until the
    /// corresponding endpoint is called.
    #[serde(default, with = "super::serde_utils::default_on_null")]
    pub required_if_supported_products: Vec<SupportedProduct>,

    /// List of Plaid product(s) that will enhance the consumer's use case but
    /// that your app can function without.
    ///
    /// Details of the products will be shown to the user in the Data
    /// Transparency Messaging consent pane, and the user may choose to opt out
    /// of sharing the data.
    #[serde(default, with = "super::serde_utils::default_on_null")]
    pub optional_products: Vec<SupportedProduct>,

    /// List of additional Plaid product(s) to collect consent for.
    ///
    /// These products will not be billed until you start using them by calling
    /// the corresponding endpoints.
    #[serde(default, with = "super::serde_utils::default_on_null")]
    pub additional_consented_products: Vec<SupportedProduct>,

    /// The destination URL to which any webhooks should be sent.
    pub webhook: Option<String>,

    /// The `access_token` associated with the Item to update or reference, used
    /// when updating, modifying, or accessing an existing `access_token`.
    ///
    /// Used when launching Link in update mode, when completing the Same-day
    /// (manual) Micro-deposit flow, or (optionally) when initializing Link for
    /// Payment Initiation.
    pub access_token: Option<String>,

    /// The name of the Link customization from the Plaid Dashboard to be
    /// applied to Link.
    ///
    /// If not specified, the default customization will be
    /// used. When using a Link customization, the language in the customization
    /// must match the language selected via the language parameter, and the
    /// countries in the customization should match the country codes selected
    /// via `country_codes`.
    pub link_customization_name: Option<String>,

    // TODO: should `redirect_uri` (and all urls) be of type `Url`?
    /// A URI indicating the destination where a user should be forwarded after
    /// completing the Link flow; used to support OAuth authentication flows
    /// when launching Link in the browser or via a webview.
    ///
    /// When used in Production, must be an https URI. If
    /// `android_package_name` is specified, this field should be left blank.
    /// For iOS integrations, `redirect_uri` should be left blank and the
    /// client-side `oauthRedirectUri` parameter should be used instead. Note
    /// that any redirect URI must also be added to the Allowed redirect URIs
    /// list in the [developer dashboard].
    ///
    /// [developer dashboard]: https://dashboard.plaid.com/team/api
    pub redirect_uri: Option<String>,

    /// The name of your app's Android package.
    ///
    /// Required if using the `link_token` to initialize Link on Android. Any
    /// package name specified here must also be added via the Allowed Android
    /// package names setting on the developer dashboard.
    pub android_package_name: Option<String>,

    /// Limits the account subtypes shown in Link.
    ///
    /// This filtering applies to both the Account Select view (if enabled) and
    /// the Institution Select view. Institutions that do not support the
    /// selected subtypes will be omitted from Link.
    pub account_filters: Option<AccountFilters>,

    /// Used for supporting legacy custom initializers.
    #[deprecated = "only used for supporting legacy custom initializers"]
    pub institution_id: Option<String>,

    /// Options for initializing Link for use with the Payment Initiation
    /// (Europe) product.
    ///
    /// *Note*: This field is required if `payment_initiation` is included in
    /// the product array.
    pub payment_initiation: Option<PaymentInitiationConfiguration>,

    /// Configuration for the Plaid-hosted Link flow.
    ///
    /// When set, the response's `hosted_link_url` will contain a URL that
    /// serves a Plaid-hosted Link session using this `link_token`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hosted_link: Option<HostedLinkConfiguration>,

    /// A `user_token` generated using `/user/create`.
    ///
    /// Any Item created during the Link session will be associated with the
    /// user. Required for Income, Multi-Item Link, and Consumer Report
    /// (CRA) flows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_token: Option<String>,
}

// Not derived: `institution_id` is deprecated, and `#[derive(Default)]` would
// fire the deprecation warning at every use site.
#[allow(clippy::derivable_impls)]
impl Default for CreateLinkTokenRequest {
    fn default() -> Self {
        #[allow(deprecated)]
        Self {
            client_name: String::new(),
            language: SupportedLanguage::default(),
            country_codes: Vec::new(),
            user: EndUser::default(),
            products: Vec::new(),
            required_if_supported_products: Vec::new(),
            optional_products: Vec::new(),
            additional_consented_products: Vec::new(),
            webhook: None,
            access_token: None,
            link_customization_name: None,
            redirect_uri: None,
            android_package_name: None,
            account_filters: None,
            institution_id: None,
            payment_initiation: None,
            hosted_link: None,
            user_token: None,
        }
    }
}

/// Configuration for the Plaid-hosted Link flow.
///
/// See [Hosted Link](https://plaid.com/docs/link/hosted-link/).
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct HostedLinkConfiguration {
    /// If provided, Plaid will send the Hosted Link URL to the end user over
    /// the given channel.
    ///
    /// Possible values: `sms`, `email`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_method: Option<String>,

    /// The URL that the user will be redirected to upon completing the Hosted
    /// Link session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_redirect_uri: Option<String>,

    /// The number of seconds that the Hosted Link URL will remain valid for,
    /// from 300 (5 minutes) to 604800 (7 days).
    ///
    /// Defaults to 900 (15 minutes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_lifetime_seconds: Option<u32>,

    /// If `true`, the Hosted Link session is opened within a mobile app.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_mobile_app: Option<bool>,
}

/// The response from performing a `create_link_token` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateLinkTokenResponse {
    /// A `link_token`, which can be supplied to Link in order to initialize it
    /// and receive a `public_token`, which can be exchanged for an
    /// `access_token`.
    pub link_token: String,

    /// The expiration date and time for the `link_token`, in ISO 8601 format.
    ///
    /// A `link_token` created to generate a `public_token` that will be
    /// exchanged for a new `access_token` expires after 4 hours.
    ///
    /// A `link_token` created for an existing Item (such as when updating an
    /// existing `access_token` by launching Link in update mode) expires after
    /// 30 minutes.
    pub expiration: chrono::DateTime<chrono::FixedOffset>,

    /// A URL of a Plaid-hosted Link flow that will use the `link_token`
    /// returned by this request.
    ///
    /// Only present if `hosted_link` was provided in the request.
    #[serde(default)]
    pub hosted_link_url: Option<String>,

    /// The `user_id` of the end user, if one was created as part of this
    /// request.
    #[serde(default)]
    pub user_id: Option<String>,

    /// A unique identifier for the request, which can be used for
    /// troubleshooting. This identifier, like all Plaid identifiers, is case
    /// sensitive.
    #[serde(default)]
    pub request_id: Option<String>,
}

/// The body for the `sandbox_create_public_token` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SandboxCreatePublicTokenRequest {
    /// The ID of the institution the `Item` will be associated with
    pub institution_id: String,

    /// The products to initially pull for the `Item`.
    ///
    /// May be any products that the specified `institution_id` supports. This
    /// array may not be empty.
    pub initial_products: Vec<SupportedProduct>,

    /// The options for configuring the `Item`.
    pub options: SandboxCreatePublicTokenRequestOptions,
}

impl Default for SandboxCreatePublicTokenRequest {
    fn default() -> Self {
        Self {
            institution_id: "ins_1".to_string(),
            initial_products: vec![SupportedProduct::Auth, SupportedProduct::Identity],
            options: SandboxCreatePublicTokenRequestOptions::default(),
        }
    }
}

/// The options for configuring the `Item`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SandboxCreatePublicTokenRequestOptions {
    /// Specify a webhook to associate with the new Item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,

    /// Test username to use for the creation of the `Sandbox` `Item`.
    ///
    /// Default: `user_good`  
    pub override_username: String,

    /// Test password to use for the creation of the `Sandbox` `Item`.
    ///
    /// Default: `pass_good`  
    pub override_password: String,
}

impl Default for SandboxCreatePublicTokenRequestOptions {
    fn default() -> Self {
        Self {
            webhook: None,
            override_username: "user_good".to_string(),
            override_password: "pass_good".to_string(),
        }
    }
}

/// The response from performing a `create_public_token` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SandboxCreatePublicTokenResponse {
    /// A `public_token` for the particular `Item` corresponding to the
    /// specified access_token
    pub public_token: String,

    /// A unique identifier for the request, which can be used for
    /// troubleshooting. This identifier, like all Plaid identifiers, is case
    /// sensitive.
    pub request_id: String,
}

/// The response from performing an `exchange_public_token` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExchangePublicTokenResponse {
    /// The access token associated with the Item data is being requested for.
    pub access_token: String,

    /// The `item_id` value of the `Item `associated with the returned
    /// `access_token`.
    pub item_id: String,

    /// A unique identifier for the request, which can be used for
    /// troubleshooting. This identifier, like all Plaid identifiers, is case
    /// sensitive.
    pub request_id: String,
}

/// The response from performing an `create_processor_token` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateProcessorTokenResponse {
    /// The `processor_token` that can then be used by the Plaid partner to
    /// make API requests.
    pub processor_token: String,

    /// A unique identifier for the request, which can be used for
    /// troubleshooting. This identifier, like all Plaid identifiers, is case
    /// sensitive.
    pub request_id: String,
}

/// Languages that Link can be displayed in.
///
/// When using a Link customization, the language configured here must match
/// the setting in the customization, or the customization will not be applied.
#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs, non_camel_case_types)]
pub enum SupportedLanguage {
    /// Danish.
    da,
    /// German.
    de,
    /// English.
    #[default]
    en,
    /// Estonian.
    et,
    /// Spanish.
    es,
    /// French.
    fr,
    /// Hindi.
    hi,
    /// Italian.
    it,
    /// Lithuanian.
    lt,
    /// Latvian.
    lv,
    /// Dutch.
    nl,
    /// Norwegian.
    no,
    /// Polish.
    pl,
    /// Portuguese.
    pt,
    /// Romanian.
    ro,
    /// Swedish.
    sv,
    /// Vietnamese.
    vi,
}

/// Countries supported by Plaid, in [ISO 3166-1 alpha-2] format.
///
/// [ISO 3166-1 alpha-2]: https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum SupportedCountry {
    /// `US`
    US,
    /// `GB`
    GB,
    /// `ES`
    ES,
    /// `NL`
    NL,
    /// `FR`
    FR,
    /// `IE`
    IE,
    /// `CA`
    CA,
    /// `DE`
    DE,
    /// `IT`
    IT,
    /// `PL`
    PL,
    /// `DK`
    DK,
    /// `NO`
    NO,
    /// `SE`
    SE,
    /// `EE`
    EE,
    /// `LT`
    LT,
    /// `LV`
    LV,
    /// `PT`
    PT,
    /// `BE`
    BE,
    /// `AT`
    AT,
    /// `FI`
    FI,
    /// A country that Plaid has added support for since this version of the
    /// crate was released.
    #[serde(other)]
    Unknown(String),
}

/// Payment processors and partners that a `processor_token` can be created for.
///
/// See [/processor/token/create].
///
/// [/processor/token/create]: https://plaid.com/docs/api/processor-partners/#processortokencreate
#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SupportedProcessor {
    /// `dwolla`
    Dwolla,
    /// `galileo`
    Galileo,
    /// `modern_treasury`
    ModernTreasury,
    /// `ocrolus`
    Ocrolus,
    /// `vesta`
    Vesta,
    /// `drivewealth`
    Drivewealth,
    /// `vopay`
    Vopay,
    /// `achq`
    Achq,
    /// `check`
    Check,
    /// `checkbook`
    Checkbook,
    /// `circle`
    Circle,
    /// `sila_money`
    SilaMoney,
    /// `rize`
    Rize,
    /// `svb_api`
    SvbApi,
    /// `unit`
    Unit,
    /// `wyre`
    Wyre,
    /// `lithic`
    Lithic,
    /// `alpaca`
    Alpaca,
    /// `astra`
    Astra,
    /// `moov`
    Moov,
    /// `treasury_prime`
    TreasuryPrime,
    /// `marqeta`
    Marqeta,
    /// `checkout`
    Checkout,
    /// `solid`
    Solid,
    /// `highnote`
    Highnote,
    /// `gemini`
    Gemini,
    /// `apex_clearing`
    ApexClearing,
    /// `gusto`
    Gusto,
    /// `adyen`
    Adyen,
    /// `atomic`
    Atomic,
    /// `i2c`
    I2c,
    /// `wepay`
    Wepay,
    /// `riskified`
    Riskified,
    /// `utb`
    Utb,
    /// `adp_roll`
    AdpRoll,
    /// `fortress_trust`
    FortressTrust,
    /// `bond`
    Bond,
    /// `bakkt`
    Bakkt,
    /// `teal`
    Teal,
    /// `zero_hash`
    ZeroHash,
    /// `taba_pay`
    TabaPay,
    /// `knot`
    Knot,
    /// `sardine`
    Sardine,
    /// `alloy`
    Alloy,
    /// `finix`
    Finix,
    /// `nuvei`
    Nuvei,
    /// `layer`
    Layer,
    /// `boom`
    Boom,
    /// `paynote`
    Paynote,
    /// `stake`
    Stake,
    /// `wedbush`
    Wedbush,
    /// `esusu`
    Esusu,
    /// `ansa`
    Ansa,
    /// `scribeup`
    Scribeup,
    /// `straddle`
    Straddle,
    /// `loanpro`
    Loanpro,
    /// `bloom_credit`
    BloomCredit,
    /// `sfox`
    Sfox,
    /// `brale`
    Brale,
    /// `parafin`
    Parafin,
    /// `cardless`
    Cardless,
    /// `open_ledger`
    OpenLedger,
    /// `valon`
    Valon,
    /// `gainbridge`
    Gainbridge,
    /// `cardlytics`
    Cardlytics,
    /// `pinwheel`
    Pinwheel,
    /// `thread_bank`
    ThreadBank,
    /// `array`
    Array,
    /// `fiant`
    Fiant,
    /// `oatfi`
    Oatfi,
    /// `curinos`
    Curinos,
    /// `frame`
    Frame,
    /// `interchecks`
    Interchecks,
    /// `interchange`
    Interchange,
    /// `atomicfi`
    Atomicfi,
    /// `pay`
    Pay,
    /// `natural`
    Natural,
    /// `kanmon`
    Kanmon,
    /// `kick`
    Kick,
    /// `increase`
    Increase,
    /// `airwallex`
    Airwallex,
    /// `cybrid`
    Cybrid,
    /// `bizcap`
    Bizcap,
    /// `webull`
    Webull,
    /// A processor that Plaid has added support for since this version of the
    /// crate was released.
    #[serde(other)]
    Unknown(String),
}

/// An object specifying information about the end user who will be linking
/// their account.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct EndUser {
    /// A unique ID representing the end user.
    ///
    /// Typically this will be a user ID number from your application.
    /// Personally identifiable information, such as an email address or phone
    /// number, should not be used in the `client_user_id`.
    pub client_user_id: String,

    /// The user's full legal name.
    ///
    /// Used for [micro-deposit based verification flows]; if the user has
    /// verified their identity, this is also used to pre-fill the name field
    /// in Identity Verification.
    ///
    /// [micro-deposit based verification flows]: https://plaid.com/docs/auth/coverage/testing/
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_name: Option<String>,

    /// The user's phone number in [E.164] format.
    ///
    /// If supplied, will be used to pre-fill the phone number field in Link.
    ///
    /// [E.164]: https://en.wikipedia.org/wiki/E.164
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,

    /// The date and time the phone number was verified, in [ISO 8601] format.
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone_number_verified_time: Option<chrono::DateTime<chrono::FixedOffset>>,

    /// The user's email address.
    ///
    /// Can be used to pre-fill Link fields when used with Identity
    /// Verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,

    /// The date and time the email address was verified, in [ISO 8601] format.
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_address_verified_time: Option<chrono::DateTime<chrono::FixedOffset>>,

    /// The user's date of birth, used to pre-fill Link fields when used with
    /// Identity Verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<chrono::NaiveDate>,
}

/// Plaid products supported by Link.
///
/// See [Plaid products](https://plaid.com/docs/api/products/).
#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SupportedProduct {
    /// `assets`
    Assets,
    /// `auth`
    Auth,
    /// `balance`
    Balance,
    /// `balance_plus`
    BalancePlus,
    /// `beacon`
    Beacon,
    /// `identity`
    Identity,
    /// `identity_match`
    IdentityMatch,
    /// `investments`
    Investments,
    /// `investments_auth`
    InvestmentsAuth,
    /// `liabilities`
    Liabilities,
    /// `payment_initiation`
    PaymentInitiation,
    /// `identity_verification`
    IdentityVerification,
    /// `transactions`
    Transactions,
    /// `credit_details`
    CreditDetails,
    /// `income`
    Income,
    /// `income_verification`
    IncomeVerification,
    /// `standing_orders`
    StandingOrders,
    /// `transfer`
    Transfer,
    /// `employment`
    Employment,
    /// `recurring_transactions`
    RecurringTransactions,
    /// `transactions_refresh`
    TransactionsRefresh,
    /// `signal`
    Signal,
    /// `statements`
    Statements,
    /// `processor_payments`
    ProcessorPayments,
    /// `processor_identity`
    ProcessorIdentity,
    /// `profile`
    Profile,
    /// `cra_base_report`
    CraBaseReport,
    /// `cra_income_insights`
    CraIncomeInsights,
    /// `cra_partner_insights`
    CraPartnerInsights,
    /// `cra_network_insights`
    CraNetworkInsights,
    /// `cra_cashflow_insights`
    CraCashflowInsights,
    /// `cra_monitoring`
    CraMonitoring,
    /// `cra_lend_score`
    CraLendScore,
    /// `cra_plaid_credit_score`
    CraPlaidCreditScore,
    /// `cra_qualify`
    CraQualify,
    /// `cra_home_lending`
    CraHomeLending,
    /// `layer`
    Layer,
    /// `pay_by_bank`
    PayByBank,
    /// `protect_linked_bank`
    ProtectLinkedBank,
    /// `protect_transactions`
    ProtectTransactions,
    /// A product that Plaid has introduced since this version of the crate was
    /// released.
    #[serde(other)]
    Unknown(String),
}

/// Options for initializing Link for use with the Payment Initiation (Europe)
/// product.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct PaymentInitiationConfiguration {
    /// The `payment_id` provided by the `/payment_initiation/payment/create`
    /// endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_id: Option<String>,

    /// The `consent_id` provided by the `/payment_initiation/consent/create`
    /// endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consent_id: Option<String>,
}

/// Limits the account subtypes shown in Link, per account type.
///
/// By default, Link will only display account types that are compatible with
/// all products supplied in the `products` parameter of `/link/token/create`.
/// Only the specified subtypes will be shown. Any account type for which a
/// filter is not specified will be entirely omitted from Link.
///
/// For institutions using OAuth, the filter will not affect the list of
/// institutions or accounts shown by the bank in the OAuth window.
///
/// See the [Account schema] for the full list of valid types and subtypes.
///
/// [Account schema]: https://plaid.com/docs/api/accounts/#account-type-schema
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct AccountFilters {
    /// Subtypes of `depository` accounts to be shown in Link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depository: Option<AccountSubtypeFilter<super::DepositorySubtype>>,

    /// Subtypes of `credit` accounts to be shown in Link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credit: Option<AccountSubtypeFilter<super::CreditSubtype>>,

    /// Subtypes of `loan` accounts to be shown in Link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loan: Option<AccountSubtypeFilter<super::LoanSubtype>>,

    /// Subtypes of `investment` accounts to be shown in Link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub investment: Option<AccountSubtypeFilter<super::InvestmentSubtype>>,

    /// Subtypes of `other` accounts to be shown in Link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other: Option<AccountSubtypeFilter<super::OtherSubtype>>,
}

/// The account subtypes to be shown in Link for a single account type.
///
/// `T` is the subtype enum belonging to that account type, so a filter can only
/// name subtypes the account type actually has.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct AccountSubtypeFilter<T> {
    /// The list of subtypes to show. Use [`SubtypeSelector::All`] to show every
    /// subtype for the account type.
    pub account_subtypes: Vec<SubtypeSelector<T>>,
}

/// A single entry in an [`AccountSubtypeFilter`]: either a specific subtype or
/// the wildcard `"all"`.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum SubtypeSelector<T> {
    /// All subtypes of the account type should be shown (`"all"`).
    #[serde(with = "super::serde_utils::strings::all")]
    All,

    /// Only this specific subtype should be shown.
    Subtype(T),
}
