use serde::{Deserialize, Serialize};

// TODO: make a `link` module?

// TODO: clean API to encode invariants of `CreateLinkTokenRequestParameters`
/// The parameters to a `create_link_token` request.
#[derive(Serialize, Debug)]
pub struct CreateLinkTokenRequestParameters {
    /// The name of your application, as it should be displayed in Link.
    pub client_name: String,

    /// The language that Link should be displayed in.
    ///
    /// When using a Link customization, the language configured here must match
    /// the setting in the customization, or the customization will not be
    /// applied.
    pub language: SupportedLanguage,

    /// Specify an array of Plaid-supported country codes using the [ISO-3166-1
    /// alpha-2] country code standard.
    ///
    /// Note that if you initialize with a European country code, your users
    /// will see the European consent panel during the Link flow. If Link is
    /// launched with multiple country codes, only products that you are enabled
    /// for in all countries will be used by Link.
    ///
    /// If using a Link customization, make sure the country codes in the
    /// customization match those specified in country_codes. If both
    /// `country_codes` and a Link customization are used, the value in
    /// `country_codes` may override the value in the customization.
    ///
    /// [ISO 3166-1 alpha-2]: https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
    pub country_codes: Vec<SupportedCountry>,

    /// A unique ID representing the end user.
    pub user: EndUser,

    // TODO: should this be ser/de with `serde_utils::default_on_null`?
    /// List of Plaid product(s) you wish to use. If launching Link in update
    /// mode, should be omitted; required otherwise.
    ///
    /// In Production, you will be billed for each product that you specify when
    /// initializing Link. Note that a product cannot be removed from an Item
    /// once the Item has been initialized with that product. To stop billing on
    /// an Item for subscription-based products, such as Liabilities,
    /// Investments, and Transactions, remove the Item via `/item/remove`.
    #[serde(default, with = "super::serde_utils::default_on_null")]
    pub products: Vec<SupportedProduct>,

    /// The destination URL to which any webhooks should be sent.
    pub webhook: Option<String>,

    /// The access_token associated with the Item to update, used when updating
    /// or modifying an existing access_token. Used when launching Link in
    /// update mode or (optionally) when initializing Link as part of the
    /// Payment Initiation (UK) flow.
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
    /// When used in Production or Development, must be an https URI. If
    /// android_package_name is specified, this field should be left blank. For
    /// iOS integrations, redirect_uri should be left blank and the client-side
    /// oauthRedirectUri parameter should be used instead. Note that any
    /// redirect URI must also be added to the Allowed redirect URIs list in the
    /// [developer dashboard].
    ///
    /// [developer dashboard]: https://dashboard.plaid.com/team/api
    pub redirect_uri: Option<String>,

    /// The name of your app's Android package.
    ///
    /// Required if using the `link_token` to initialize Link on Android. Any
    /// package name specified here must also be added via the Allowed Android
    /// package names setting on the developer dashboard.
    pub android_package_name: Option<String>,

    // TODO: figure out `Account{Sub}Type` and make `account_filters` not a `Map`
    /// By default, Link will only display account types that are compatible
    /// with all products supplied in the products parameter of
    /// `/link/token/create`. You can further limit the accounts shown in Link
    /// by using `account_filters` to specify the account subtypes to be shown
    /// in Link. Only the specified subtypes will be shown. This filtering
    /// applies to both the Account Select view (if enabled) and the Institution
    /// Select view. Institutions that do not support the selected subtypes will
    /// be omitted from Link. To indicate that all subtypes should be shown, use
    /// the value `"all"`. If the account_filters filter is used, any account
    /// type for which a filter is not specified will be entirely omitted from
    /// Link.
    ///
    /// Example value:
    /// ```json
    /// {
    ///   "depository": {
    ///     "account_subtypes": ["checking", "savings"]
    ///   },
    ///   "credit": {
    ///     "account_subtypes": ["all"]
    ///   }
    /// }
    /// ```
    /// For a full list of valid types and subtypes, see the [Account schema].
    ///
    /// For institutions using OAuth, the filter will not affect the list of
    /// institutions or accounts shown by the bank in the OAuth window.
    ///
    /// [Account schema]: https://plaid.com/docs/api/accounts#accounts-schema
    pub account_filters: serde_json::Map<String, serde_json::Value>,

    /// Used for supporting legacy custom initializers.
    #[deprecated = "only used for supporting legacy custom initializers"]
    pub institution_id: Option<String>,

    /// Options for initializing Link for use with the Payment Initiation (UK)
    /// product.
    ///
    /// *Note*: This field is required if `payment_initiation` is included in
    /// the product array.
    pub payment_initiation: Option<PaymentInitiationConfiguration>,
}

/// The response from performing a `create_link_token` request.
#[derive(Deserialize, Debug)]
pub struct CreateLinkTokenResponse {
    /// A `link_token`, which can be supplied to Link in order to initialize it
    /// and receive a public_token, which can be exchanged for an
    /// `access_token`.
    pub link_token: String,

    /// The expiration date for the `link_token`, in ISO 8601 format.
    ///
    /// A `link_token` created to generate a `public_token` that will be
    /// exchanged for a new `access_token` expires after 24 hours.
    ///
    /// A `link_token` created for an existing Item (such as when updating an
    /// existing access_token by launching Link in update mode) expires after 30
    /// minutes.
    pub expiration: chrono::DateTime<chrono::FixedOffset>,
}

/// The response from performing a `create_public_token` request.
#[derive(Deserialize, Debug)]
pub struct CreatePublicTokenResponse {
    /// A `public_token` for the particular `Item` corresponding to the
    /// specified access_token
    pub public_token: String,

    /// A unique identifier for the request, which can be used for
    /// troubleshooting. This identifier, like all Plaid identifiers, is case
    /// sensitive.
    pub request_id: String,
}

/// The response from performing an `exchange_public_token` request.
#[derive(Deserialize, Debug)]
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

/// Supported languages.
#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
#[allow(missing_docs, non_camel_case_types)]
pub enum SupportedLanguage {
    en,
    fr,
    es,
    nl,
}

/// Supported countries in [ISO 3166-1 alpha-2] format.
///
/// [ISO 3166-1 alpha-2]: https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
#[derive(Serialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
#[allow(missing_docs)]
pub enum SupportedCountry {
    US,
    CA,
    ES,
    FR,
    GB,
    IE,
    NL,
}

/// An object specifying information about the end user who will be linking
/// their account.
#[derive(Serialize, Debug)]
pub struct EndUser {
    /// A unique ID representing the end user.
    ///
    /// Typically this will be a user ID number from your application.
    /// Personally identifiable information, such as an email address or phone
    /// number, should not be used in the `client_user_id`.
    client_user_id: String,
}

/// Plaid product supported by Link.
///
/// *Note*: `Balance` is not a valid value, the Balance product does not require
/// explicit initalization and will automatically be initialized when any other
/// product is initialized.
#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum SupportedProduct {
    Transactions,
    Auth,
    Identity,
    Assets,
    Investments,
    Liabilities,
    PaymentInitiation,
}

/// Options for initializing Link for use with the Payment Initiation
/// (UK) product.
#[derive(Serialize, Debug)]
pub struct PaymentInitiationConfiguration {
    /// The `payment_id` provided by the `/payment_initiation/payment/create`
    /// endpoint.
    payment_id: String,
}
