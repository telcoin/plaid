//! Account types.

use serde::{Deserialize, Serialize};

/// The response from performing an `accounts` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccountsResponse {
    /// The financial institution accounts associated with the Item.
    #[serde(default)]
    pub accounts: Vec<Account>,

    /// Metadata about the Item.
    pub item: super::Item,

    /// A unique identifier for the request, which can be used for
    /// troubleshooting. This identifier, like all Plaid identifiers, is case
    /// sensitive.
    pub request_id: String,
}

/// Financial institution accounts associated with the `Item`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    /// Plaid’s unique identifier for the account. This value will not change
    /// unless Plaid can't reconcile the account with the data returned by the
    /// financial institution. This may occur, for example, when the name of the
    /// account changes. If this happens a new `account_id` will be assigned to
    /// the account.
    ///
    /// The `account_id` can also change if the `access_token` is deleted and
    /// the same credentials that were used to generate that access_token are
    /// used to generate a new `access_token` on a later date. In that case, the
    /// new `account_id` will be different from the old `account_id`.
    ///
    /// Like all Plaid identifiers, the account_id is case sensitive.
    pub account_id: String,

    /// A set of fields describing the balance for an account.
    ///
    /// Available and current balance information may be cached and is not
    /// guaranteed to be up-to-date in realtime unless the balance object was
    /// returned by `/account/balance/get`.
    pub balances: Balances,

    /// The last 2-4 alphanumeric characters of an account's official account
    /// number. Note that the mask may be non-unique between an `Item`'s
    /// accounts, and it may also not match the mask that the bank displays to
    /// the user.
    pub mask: Option<String>,

    /// The name of the account, either assigned by the user or by the financial
    /// institution itself
    pub name: String,

    /// The official name of the account as given by the financial institution
    pub official_name: Option<String>,

    /// The type of this `Account`.
    #[serde(rename = "type")]
    pub ty: AccountType,

    /// The current verification status of this `Account`.
    pub verification_status: Option<VerificationStatus>,

    /// Calculated data about the historical balances on the account.
    ///
    /// Only returned by Assets endpoints.
    #[serde(default)]
    pub historical_balances: Vec<HistoricalBalance>,

    /// Data returned by the financial institution about the account owner or
    /// owners.
    ///
    /// Only returned by Identity or Assets endpoints. Multiple owners on a
    /// single account will be represented in the same owner object, not in
    /// multiple owner objects within the array.
    #[serde(default)]
    pub owners: Vec<Owner>,

    /// The duration of transaction history available for this Item, typically
    /// defined as the time since the date of the earliest transaction in that
    /// account.
    ///
    /// Only returned by Assets endpoints.
    pub days_available: Option<u32>,
}

// TODO: use a money crate
// TODO: use tagged enum instead of both currency fields
/// A set of fields describing the balance for an account.
///
/// Available and current balance information may be cached and is not
/// guaranteed to be up-to-date in realtime unless the balance object was
/// returned by `/account/balance/get`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Balances {
    /// The amount of funds available to be withdrawn from the account, as
    /// determined by the financial institution.
    ///
    /// For credit-type accounts, the available balance typically equals the
    /// limit less the current balance, less any pending outflows plus any
    /// pending inflows.
    ///
    /// For depository-type accounts, the available balance typically equals the
    /// current balance less any pending outflows plus any pending inflows. For
    /// depository-type accounts, the available balance does not include the
    /// overdraft limit.
    ///
    /// For investment-type accounts, the available balance is the total cash
    /// available to withdraw as presented by the institution.
    ///
    /// Note that not all institutions calculate the available balance. In the
    /// event that available balance is unavailable, Plaid will return an
    /// available balance value of null.
    ///
    /// Available balance may be cached and is not guaranteed to be up-to-date
    /// in realtime unless the value was returned by `/account/balance/get`.
    pub available: Option<f64>,

    /// The total amount of funds in or owed by the account.
    ///
    /// For credit-type accounts, a positive balance indicates the amount owed;
    /// a negative amount indicates the lender owing the account holder.
    ///
    /// For loan-type accounts, the current balance is the principal remaining
    /// on the loan.
    ///
    /// For investment-type accounts, the current balance is the total value of
    /// assets as presented by the institution.
    ///
    /// Current balance may be cached and is not guaranteed to be up-to-date in
    /// realtime unless the value was returned by `/account/balance/get`.
    pub current: f64,

    /// For credit-type accounts, this represents the credit limit.
    ///
    /// For depository-type accounts, this represents the pre-arranged overdraft
    /// limit, which is common for current (checking) accounts in Europe.
    ///
    /// In North America, this field is typically only available for credit-type
    /// accounts.
    pub limit: Option<f64>,

    // TODO: use ISO 4217 library
    /// The [ISO 4217] currency code of the balance.
    ///
    /// Always null if `unofficial_currency_code` is non-null.
    ///
    /// [ISO 4217]: https://en.wikipedia.org/wiki/ISO_4217
    pub iso_currency_code: Option<String>,

    /// The unofficial currency code associated with the balance.
    ///
    /// Always null if `iso_currency_code` is non-null.
    pub unofficial_currency_code: Option<String>,
}

// TODO: use tagged enum instead of both currency fields
/// An account balance from a specific point in time.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoricalBalance {
    /// The date of the calculated historical balance.
    pub date: chrono::NaiveDate,

    /// The total amount of funds in the account, calculated from the current
    /// balance in the balance object by subtracting inflows and adding back
    /// outflows according to the posted date of each transaction.
    ///
    /// If the account has any pending transactions, historical balance amounts
    /// on or after the date of the earliest pending transaction may differ if
    /// retrieved in subsequent Asset Reports as a result of those pending
    /// transactions posting.
    pub current: String,

    // TODO: use ISO 4217 library
    /// The [ISO 4217] currency code of the balance.
    ///
    /// Always null if `unofficial_currency_code` is non-null.
    ///
    /// [ISO 4217]: https://en.wikipedia.org/wiki/ISO_4217
    pub iso_currency_code: Option<String>,

    /// The unofficial currency code associated with the balance.
    ///
    /// Always null if `iso_currency_code` is non-null.
    pub unofficial_currency_code: Option<String>,
}

/// Account holder(s) information.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Owner {
    /// A list of names associated with the account by the financial
    /// institution. These should always be the names of individuals, even for
    /// business accounts. If the name of a business is reported, please contact
    /// Plaid Support. In the case of a joint account, the names of all account
    /// holders will be reported.
    ///
    /// If an Item contains multiple accounts with different owner names, some
    /// institutions will report all names associated with the Item in each
    /// account's `names` array.
    #[serde(default)]
    pub names: Vec<String>,

    /// A list of phone numbers associated with the account by the financial
    /// institution.
    ///
    /// May be an empty array if no relevant information is /// returned from
    /// the financial institution.
    #[serde(default)]
    pub phone_numbers: Vec<PhoneNumber>,

    /// A list of email addresses associated with the account by the financial
    /// institution.
    ///
    /// May be an empty array if no relevant information is returned from the
    /// financial institution.
    #[serde(default)]
    pub emails: Vec<EmailAddress>,

    /// Data about the various addresses associated with the account by the
    /// financial institution.
    ///
    /// May be an empty array if no relevant information is returned from the
    /// financial institution.
    #[serde(default)]
    pub addresses: Vec<Address>,
}

/// Details Phone number associated with an `Account`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhoneNumber {
    /// The phone number.
    pub data: String,

    /// When `true`, identifies the phone number as the primary number on an
    /// account.
    pub primary: Option<bool>,

    // TODO: should this be a string instead? see HACK
    /// The type of phone number.
    #[serde(rename = "type")]
    pub ty: Option<PhoneNumberType>,
}

/// The type of phone number
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged, rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum PhoneNumberType {
    #[serde(with = "super::serde_utils::strings::home")]
    Home,
    #[serde(with = "super::serde_utils::strings::work")]
    Work,
    #[serde(with = "super::serde_utils::strings::office")]
    Office,
    #[serde(with = "super::serde_utils::strings::mobile")]
    Mobile,
    // TODO: should we keep this variant?
    #[serde(with = "super::serde_utils::strings::mobile1")]
    Mobile1,
    // collect all other types in `Other`
    Other(String),
}

/// An email address associated with this `Account`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmailAddress {
    /// The email address.
    pub data: String,

    /// When `true`, identifies the email address as the primary email on an
    /// account.
    pub primary: bool,

    /// The type of email account as described by the financial institution.
    #[serde(rename = "type")]
    pub ty: EmailAddressType,
}

/// The type of email account as described by the financial institution.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum EmailAddressType {
    Primary,
    Secondary,
    Other,
}

/// A physical address associated with the account by the financial institution.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Address {
    /// Data about the components comprising an address.
    pub data: AddressDetails,

    /// When `true`, identifies the address as the primary address on an
    /// account.
    pub primary: Option<bool>,
}

/// The actual address details.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddressDetails {
    /// The full city name
    pub city: Option<String>,

    /// The region or state Example: `"NC"`
    pub region: Option<String>,

    /// The full street address Example: `"564 Main Street, APT 15"`
    pub street: String,

    /// The postal code
    pub postal_code: Option<String>,

    // TODO: this is not optional according to the docs, but it is `null` in test data
    // TODO: make country an enum/use crate
    /// The [ISO 3166-1 alpha-2] country code
    ///
    /// [ISO 3166-1 alpha-2]: https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
    pub country: Option<String>,
}

// TODO: add account sub-types; how do we handle ser/de?
/// Account types.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// An account type holding cash, in which funds are deposited.
    ///
    /// Supported products for depository accounts are: Auth, Balance,
    /// Transactions, Identity, Payment Initiation, and Assets.
    Depository,

    /// A credit card type account.
    ///
    /// Supported products for credit accounts are: Balance, Transactions,
    /// Identity, and Liabilities.
    Credit,

    /// A loan type account.
    ///
    /// Supported products for loan accounts are: Balance, Liabilities, and
    /// Transactions.
    Loan,

    /// An investment account.
    ///
    /// Supported products for investment accounts are: Balance and Investments.
    Investment,

    /// Other or unknown account type.
    ///
    /// Supported products for other accounts are: Balance, Transactions,
    /// Identity, and Assets.
    Other,
}

/// The current verification status of an Auth Item initiated through Automated
/// or Manual micro-deposits. Returned for Auth Items only.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    /// The Item is pending automatic verification
    PendingAutomaticVerification,

    /// The Item is pending manual micro-deposit verification. Items remain in
    /// this state until the user successfully verifies the two amounts.
    PendingManualVerification,

    /// The Item has successfully been automatically verified
    AutomaticallyVerified,

    /// The Item has successfully been manually verified
    ManuallyVerified,

    /// Plaid was unable to automatically verify the deposit within 7 calendar
    /// days and will no longer attempt to validate the Item. Users may retry by
    /// submitting their information again through Link.
    VerificationExpired,
}

/// Options for the `balance` request.
#[derive(Serialize, Default, Clone, Debug)]
pub struct BalanceRequestOptions {
    /// A list of `account_ids` to retrieve for the Item.
    ///
    /// *Note*: An error will be returned if a provided `account_id` is not
    /// associated with the Item.
    #[serde(default, with = "super::serde_utils::default_on_null")]
    pub account_ids: Vec<String>,
}
