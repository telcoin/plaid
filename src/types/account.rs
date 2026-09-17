//! Account types.

use serde::{Deserialize, Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

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

    /// The type of this `Account`, together with its subtype.
    ///
    /// The API sends these as two fields, `type` and `subtype`, but a subtype
    /// is only meaningful for one particular type — so they are modelled here
    /// as one value whose subtype is scoped to the type that owns it.
    #[serde(flatten)]
    pub ty: AccountType,

    /// The current verification status of this `Account`.
    pub verification_status: Option<VerificationStatus>,

    /// The account holder name that was used for micro-deposit and/or database
    /// verification.
    ///
    /// Only returned for Auth Items created via micro-deposit or database
    /// verification. Note that this name is a copy of the name that was
    /// submitted, and may not always match the name on the account.
    #[serde(default)]
    pub verification_name: Option<String>,

    /// A unique and persistent identifier for accounts that can be used to
    /// trace multiple instances of the same account across different Items for
    /// depository accounts.
    ///
    /// This field is currently supported only for Items at institutions that
    /// use Tokenized Account Numbers (i.e. Chase and PNC) and Items created via
    /// Same Day Micro-deposits.
    #[serde(default)]
    pub persistent_account_id: Option<String>,

    /// The annual percentage yield (APY) on an interest-bearing deposit
    /// account, expressed as a percentage.
    ///
    /// Only available for `depository` accounts at select institutions, and
    /// only when the Balance Plus product is enabled.
    #[serde(default)]
    pub apy: Option<f64>,

    /// The account holder category: whether the account is a business or a
    /// personal account.
    ///
    /// Only available for `depository` and `credit` accounts at select
    /// institutions.
    #[serde(default)]
    pub holder_category: Option<AccountHolderCategory>,

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
    ///
    /// If current is null this field is guaranteed not to be null.
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
    ///
    /// When returned by /accounts/balance/get, this field may be null. When
    /// this happens, available is guaranteed not to be null.
    pub current: Option<f64>,

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

    /// Timestamp in [ISO 8601] format (`YYYY-MM-DDTHH:mm:ssZ`) indicating the
    /// last time that the balance for the given account has been updated.
    ///
    /// This is currently only provided when the `min_last_updated_datetime`
    /// option is passed to `/accounts/balance/get`.
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    #[serde(default)]
    pub last_updated_datetime: Option<chrono::DateTime<chrono::FixedOffset>>,
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
#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PhoneNumberType {
    /// `home`
    Home,
    /// `work`
    Work,
    /// `office`
    Office,
    /// `mobile`
    Mobile,
    /// `mobile1`
    Mobile1,
    /// `other`
    Other,
    /// A type Plaid has introduced since this version of the crate was
    /// released.
    #[serde(other)]
    Unknown(String),
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

/// The type of an `Account`, together with the subtype that belongs to it.
///
/// Plaid sends `type` and `subtype` as two independent fields, but each subtype
/// is only ever valid for one type. Pairing them here means a mismatched
/// combination cannot be constructed, and matching on the type gives you a
/// subtype already narrowed to that type's possibilities.
///
/// The subtype is `None` when Plaid could not determine one.
///
/// See the [Account schema].
///
/// [Account schema]: https://plaid.com/docs/api/accounts/#account-type-schema
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(from = "AccountTypeRepr", into = "AccountTypeRepr")]
pub enum AccountType {
    /// An account type holding cash, in which funds are deposited.
    ///
    /// Supported products for depository accounts are: Auth, Balance,
    /// Transactions, Identity, Payment Initiation, and Assets.
    Depository(Option<DepositorySubtype>),

    /// A credit card type account.
    ///
    /// Supported products for credit accounts are: Balance, Transactions,
    /// Identity, and Liabilities.
    Credit(Option<CreditSubtype>),

    /// A loan type account.
    ///
    /// Supported products for loan accounts are: Balance, Liabilities, and
    /// Transactions.
    Loan(Option<LoanSubtype>),

    /// An investment account.
    ///
    /// Supported products for investment accounts are: Balance and Investments.
    Investment(Option<InvestmentSubtype>),

    /// An investment account held outside the US.
    ///
    /// Non-US investment accounts are reported as `brokerage`; US investment
    /// accounts are reported as `investment`.
    Brokerage(Option<BrokerageSubtype>),

    /// Other or unknown account type.
    ///
    /// Supported products for other accounts are: Balance, Transactions,
    /// Identity, and Assets.
    Other(Option<OtherSubtype>),

    /// A type Plaid has introduced since this version of the crate was
    /// released, preserved verbatim so it round-trips unchanged.
    Unknown {
        /// The raw `type` value.
        ty: String,
        /// The raw `subtype` value.
        subtype: Option<String>,
    },
}

impl AccountType {
    /// The value the account type is represented by on the wire.
    pub fn as_str(&self) -> &str {
        match self {
            AccountType::Depository(_) => "depository",
            AccountType::Credit(_) => "credit",
            AccountType::Loan(_) => "loan",
            AccountType::Investment(_) => "investment",
            AccountType::Brokerage(_) => "brokerage",
            AccountType::Other(_) => "other",
            AccountType::Unknown { ty, .. } => ty.as_str(),
        }
    }

    /// The value the account subtype is represented by on the wire, if Plaid
    /// reported one.
    pub fn subtype_str(&self) -> Option<String> {
        AccountTypeRepr::from(self.clone()).subtype
    }
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.subtype_str() {
            Some(subtype) => write!(f, "{} ({})", self.as_str(), subtype),
            None => f.write_str(self.as_str()),
        }
    }
}

/// The wire representation of [`AccountType`]: the API's separate `type` and
/// `subtype` fields.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct AccountTypeRepr {
    #[serde(rename = "type")]
    ty: String,
    #[serde(default)]
    subtype: Option<String>,
}

impl From<AccountTypeRepr> for AccountType {
    fn from(repr: AccountTypeRepr) -> Self {
        // Parsing a subtype never fails: every subtype enum has an `Unknown`
        // catch-all, so an unrecognised value is preserved rather than rejected.
        fn subtype<T>(raw: Option<&str>) -> Option<T>
        where
            T: std::str::FromStr,
            T::Err: std::fmt::Debug,
        {
            raw.map(|raw| raw.parse().expect("subtype enums are infallible"))
        }

        let raw = repr.subtype.as_deref();
        match repr.ty.as_str() {
            "depository" => AccountType::Depository(subtype(raw)),
            "credit" => AccountType::Credit(subtype(raw)),
            "loan" => AccountType::Loan(subtype(raw)),
            "investment" => AccountType::Investment(subtype(raw)),
            "brokerage" => AccountType::Brokerage(subtype(raw)),
            "other" => AccountType::Other(subtype(raw)),
            _ => AccountType::Unknown {
                ty: repr.ty,
                subtype: repr.subtype,
            },
        }
    }
}

impl From<AccountType> for AccountTypeRepr {
    fn from(ty: AccountType) -> Self {
        fn repr<T: ToString>(ty: &str, subtype: Option<T>) -> AccountTypeRepr {
            AccountTypeRepr {
                ty: ty.to_string(),
                subtype: subtype.map(|s| s.to_string()),
            }
        }

        match ty {
            AccountType::Depository(s) => repr("depository", s),
            AccountType::Credit(s) => repr("credit", s),
            AccountType::Loan(s) => repr("loan", s),
            AccountType::Investment(s) => repr("investment", s),
            AccountType::Brokerage(s) => repr("brokerage", s),
            AccountType::Other(s) => repr("other", s),
            AccountType::Unknown { ty, subtype } => AccountTypeRepr { ty, subtype },
        }
    }
}

/// Subtypes valid for an [`AccountType::Depository`] account.
///
/// See the [Account schema].
///
/// [`AccountType::Depository`]: enum.AccountType.html#variant.Depository
/// [Account schema]: https://plaid.com/docs/api/accounts/#account-type-schema
#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DepositorySubtype {
    /// `cash management`
    #[serde(rename = "cash management")]
    CashManagement,
    /// `cd`
    Cd,
    /// `checking`
    Checking,
    /// `ebt`
    Ebt,
    /// `hsa`
    Hsa,
    /// `limited purpose checking`
    #[serde(rename = "limited purpose checking")]
    LimitedPurposeChecking,
    /// `money market`
    #[serde(rename = "money market")]
    MoneyMarket,
    /// `paypal`
    Paypal,
    /// `prepaid`
    Prepaid,
    /// `savings`
    Savings,
    /// A subtype Plaid has introduced since this version of the crate was
    /// released.
    #[serde(other)]
    Unknown(String),
}

/// Subtypes valid for an [`AccountType::Credit`] account.
///
/// See the [Account schema].
///
/// [`AccountType::Credit`]: enum.AccountType.html#variant.Credit
/// [Account schema]: https://plaid.com/docs/api/accounts/#account-type-schema
#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CreditSubtype {
    /// `charge card`
    #[serde(rename = "charge card")]
    ChargeCard,
    /// `credit card`
    #[serde(rename = "credit card")]
    CreditCard,
    /// `paypal`
    Paypal,
    /// A subtype Plaid has introduced since this version of the crate was
    /// released.
    #[serde(other)]
    Unknown(String),
}

/// Subtypes valid for an [`AccountType::Loan`] account.
///
/// See the [Account schema].
///
/// [`AccountType::Loan`]: enum.AccountType.html#variant.Loan
/// [Account schema]: https://plaid.com/docs/api/accounts/#account-type-schema
#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LoanSubtype {
    /// `auto`
    Auto,
    /// `business`
    Business,
    /// `commercial`
    Commercial,
    /// `commercial line of credit`
    #[serde(rename = "commercial line of credit")]
    CommercialLineOfCredit,
    /// `construction`
    Construction,
    /// `consumer`
    Consumer,
    /// `home equity`
    #[serde(rename = "home equity")]
    HomeEquity,
    /// `home equity loan`
    #[serde(rename = "home equity loan")]
    HomeEquityLoan,
    /// `installment`
    Installment,
    /// `line of credit`
    #[serde(rename = "line of credit")]
    LineOfCredit,
    /// `loan`
    Loan,
    /// `mortgage`
    Mortgage,
    /// `other`
    Other,
    /// `overdraft`
    Overdraft,
    /// `student`
    Student,
    /// A subtype Plaid has introduced since this version of the crate was
    /// released.
    #[serde(other)]
    Unknown(String),
}

/// Subtypes valid for an [`AccountType::Investment`] account.
///
/// See the [Account schema].
///
/// [`AccountType::Investment`]: enum.AccountType.html#variant.Investment
/// [Account schema]: https://plaid.com/docs/api/accounts/#account-type-schema
#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[allow(non_camel_case_types)]
pub enum InvestmentSubtype {
    /// `401a`
    #[serde(rename = "401a")]
    _401a,
    /// `401k`
    #[serde(rename = "401k")]
    _401k,
    /// `403B`
    #[serde(rename = "403B")]
    _403b,
    /// `457b`
    #[serde(rename = "457b")]
    _457b,
    /// `529`
    #[serde(rename = "529")]
    _529,
    /// `brokerage`
    Brokerage,
    /// `cash isa`
    #[serde(rename = "cash isa")]
    CashIsa,
    /// `crypto exchange`
    #[serde(rename = "crypto exchange")]
    CryptoExchange,
    /// `education savings account`
    #[serde(rename = "education savings account")]
    EducationSavingsAccount,
    /// `fhsa`
    Fhsa,
    /// `fixed annuity`
    #[serde(rename = "fixed annuity")]
    FixedAnnuity,
    /// `gic`
    Gic,
    /// `health reimbursement arrangement`
    #[serde(rename = "health reimbursement arrangement")]
    HealthReimbursementArrangement,
    /// `hsa`
    Hsa,
    /// `ira`
    Ira,
    /// `isa`
    Isa,
    /// `keogh`
    Keogh,
    /// `lif`
    Lif,
    /// `life insurance`
    #[serde(rename = "life insurance")]
    LifeInsurance,
    /// `line of credit`
    #[serde(rename = "line of credit")]
    LineOfCredit,
    /// `lira`
    Lira,
    /// `lrif`
    Lrif,
    /// `lrsp`
    Lrsp,
    /// `mutual fund`
    #[serde(rename = "mutual fund")]
    MutualFund,
    /// `non-custodial wallet`
    #[serde(rename = "non-custodial wallet")]
    NonCustodialWallet,
    /// `non-taxable brokerage account`
    #[serde(rename = "non-taxable brokerage account")]
    NonTaxableBrokerageAccount,
    /// `other`
    Other,
    /// `other annuity`
    #[serde(rename = "other annuity")]
    OtherAnnuity,
    /// `other insurance`
    #[serde(rename = "other insurance")]
    OtherInsurance,
    /// `pension`
    Pension,
    /// `prediction market`
    #[serde(rename = "prediction market")]
    PredictionMarket,
    /// `prif`
    Prif,
    /// `profit sharing plan`
    #[serde(rename = "profit sharing plan")]
    ProfitSharingPlan,
    /// `qshr`
    Qshr,
    /// `rdsp`
    Rdsp,
    /// `resp`
    Resp,
    /// `retirement`
    Retirement,
    /// `rlif`
    Rlif,
    /// `roth`
    Roth,
    /// `roth 401k`
    #[serde(rename = "roth 401k")]
    Roth401k,
    /// `roth 403B`
    #[serde(rename = "roth 403B")]
    Roth403b,
    /// `roth 457b`
    #[serde(rename = "roth 457b")]
    Roth457b,
    /// `roth pension`
    #[serde(rename = "roth pension")]
    RothPension,
    /// `roth profit sharing plan`
    #[serde(rename = "roth profit sharing plan")]
    RothProfitSharingPlan,
    /// `roth thrift savings plan`
    #[serde(rename = "roth thrift savings plan")]
    RothThriftSavingsPlan,
    /// `rrif`
    Rrif,
    /// `rrsp`
    Rrsp,
    /// `sarsep`
    Sarsep,
    /// `sep ira`
    #[serde(rename = "sep ira")]
    SepIra,
    /// `simple ira`
    #[serde(rename = "simple ira")]
    SimpleIra,
    /// `sipp`
    Sipp,
    /// `stock plan`
    #[serde(rename = "stock plan")]
    StockPlan,
    /// `tfsa`
    Tfsa,
    /// `thrift savings plan`
    #[serde(rename = "thrift savings plan")]
    ThriftSavingsPlan,
    /// `trust`
    Trust,
    /// `ugma`
    Ugma,
    /// `utma`
    Utma,
    /// `variable annuity`
    #[serde(rename = "variable annuity")]
    VariableAnnuity,
    /// A subtype Plaid has introduced since this version of the crate was
    /// released.
    #[serde(other)]
    Unknown(String),
}

/// Subtypes valid for an [`AccountType::Brokerage`] account.
///
/// See the [Account schema].
///
/// [`AccountType::Brokerage`]: enum.AccountType.html#variant.Brokerage
/// [Account schema]: https://plaid.com/docs/api/accounts/#account-type-schema
#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BrokerageSubtype {
    /// `brokerage`
    Brokerage,
    /// `non-custodial wallet`
    #[serde(rename = "non-custodial wallet")]
    NonCustodialWallet,
    /// `non-taxable brokerage account`
    #[serde(rename = "non-taxable brokerage account")]
    NonTaxableBrokerageAccount,
    /// A subtype Plaid has introduced since this version of the crate was
    /// released.
    #[serde(other)]
    Unknown(String),
}

/// Subtypes valid for an [`AccountType::Other`] account.
///
/// See the [Account schema].
///
/// [`AccountType::Other`]: enum.AccountType.html#variant.Other
/// [Account schema]: https://plaid.com/docs/api/accounts/#account-type-schema
#[derive(Deserialize_enum_str, Serialize_enum_str, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OtherSubtype {
    /// `other`
    Other,
    /// `payroll`
    Payroll,
    /// A subtype Plaid has introduced since this version of the crate was
    /// released.
    #[serde(other)]
    Unknown(String),
}

/// Indicates an `Item`'s micro-deposit-based verification or database
/// verification status.
///
/// This field is only populated for Items that were created using
/// micro-deposit-based verification flows or database verification flows.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    /// The Item is pending automatic verification.
    PendingAutomaticVerification,

    /// The Item is pending manual micro-deposit verification. Items remain in
    /// this state until the user successfully verifies the code.
    PendingManualVerification,

    /// The micro-deposit verification code has not yet been sent to the user.
    Unsent,

    /// The Item has successfully been automatically verified.
    AutomaticallyVerified,

    /// The Item has successfully been manually verified.
    ManuallyVerified,

    /// Plaid was unable to automatically verify the deposit within 7 calendar
    /// days and will no longer attempt to validate the Item. Users may retry by
    /// submitting their information again through Link.
    VerificationExpired,

    /// The user failed manual micro-deposit verification because they exhausted
    /// all 3 verification attempts. Users may retry by submitting their
    /// information again through Link.
    VerificationFailed,

    /// The Item's ACH numbers have been verified using Plaid's Database Match
    /// product.
    DatabaseMatched,

    /// The Item's ACH numbers have been verified using Plaid's Database
    /// Insights product, and have passed.
    DatabaseInsightsPass,

    /// The Item's ACH numbers have been verified using Plaid's Database
    /// Insights product, and have passed with caution.
    DatabaseInsightsPassWithCaution,

    /// The Item's ACH numbers have been verified using Plaid's Database
    /// Insights product, and have failed.
    DatabaseInsightsFail,
}

/// The account holder category: whether an account is a business or a personal
/// account.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountHolderCategory {
    /// The account is associated with a business.
    Business,

    /// The account is associated with an individual.
    Personal,

    /// The account category could not be determined.
    Unrecognized,
}

/// Options for the `balance` request.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct BalanceRequestOptions {
    /// A list of `account_ids` to retrieve for the Item.
    ///
    /// *Note*: An error will be returned if a provided `account_id` is not
    /// associated with the Item.
    #[serde(default, with = "super::serde_utils::default_on_null")]
    pub account_ids: Vec<String>,

    /// Timestamp in [ISO 8601] format (`YYYY-MM-DDTHH:mm:ssZ`) indicating the
    /// oldest acceptable balance when making a request to
    /// `/accounts/balance/get`.
    ///
    /// If the balance that is pulled is older than the given timestamp for
    /// institutions with menu-based selection or for Capital One (`ins_128026`),
    /// an `INVALID_REQUEST` error with the code of
    /// `LAST_UPDATED_DATETIME_OUT_OF_RANGE` will be returned with the most
    /// recent timestamp for the requested account contained in the response.
    ///
    /// This field is only used when the institution is `ins_128026` (Capital
    /// One), in which case a value must be provided or an `INVALID_REQUEST`
    /// error with the code of `INVALID_FIELD` will be returned. For all other
    /// institutions, this field is ignored.
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_last_updated_datetime: Option<chrono::DateTime<chrono::FixedOffset>>,
}

/// Options for the `accounts` request.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct AccountsRequestOptions {
    /// A list of `account_ids` to retrieve for the Item.
    ///
    /// *Note*: An error will be returned if a provided `account_id` is not
    /// associated with the Item.
    #[serde(default, with = "super::serde_utils::default_on_null")]
    pub account_ids: Vec<String>,
}
