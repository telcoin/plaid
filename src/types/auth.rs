//! Auth types.

use serde::{Deserialize, Serialize};

/// Options for the `auth` request.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct AuthRequestOptions {
    /// A list of `account_ids` to retrieve for the Item.
    ///
    /// *Note*: An error will be returned if a provided `account_id` is not
    /// associated with the Item.
    #[serde(default, with = "super::serde_utils::default_on_null")]
    pub account_ids: Vec<String>,
}

/// The response from performing an `auth` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthResponse {
    /// The accounts for which numbers are being retrieved.
    #[serde(default)]
    pub accounts: Vec<super::Account>,

    /// An object containing identifying numbers used for making electronic
    /// transfers to and from the accounts.
    pub numbers: AccountNumbers,

    /// Metadata about the Item.
    pub item: super::Item,

    /// A unique identifier for the request, which can be used for
    /// troubleshooting. This identifier, like all Plaid identifiers, is case
    /// sensitive.
    pub request_id: String,
}

/// An object containing identifying numbers used for making electronic
/// transfers to and from the accounts.
///
/// The identifying number type (ACH, EFT, IBAN, or BACS) used will depend on
/// the country of the account. An account may have more than one number type.
/// If a particular identifying number type is not used by any accounts for
/// which data has been requested, the `Vec` for that type will be empty.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AccountNumbers {
    /// A list of ACH numbers identifying accounts.
    #[serde(default)]
    pub ach: Vec<AchAccountNumbers>,

    /// A list of EFT numbers identifying accounts.
    #[serde(default)]
    pub eft: Vec<EftAccountNumbers>,

    /// A list of IBAN numbers identifying accounts.
    #[serde(default)]
    pub international: Vec<InternationalAccountNumbers>,

    /// A list of BACS numbers identifying accounts.
    #[serde(default)]
    pub bacs: Vec<BacsAccountNumbers>,
}

/// The numbers identifying an ACH account.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AchAccountNumbers {
    /// The Plaid account ID associated with the account numbers
    pub account_id: String,

    /// The ACH account number for the account.
    ///
    /// At certain institutions, including Chase and PNC, you will receive a
    /// "tokenized" routing and account number pair, which is not the user's
    /// actual account and routing number. See
    /// [`is_tokenized_account_number`](#structfield.is_tokenized_account_number)
    /// and Plaid's [Tokenized account numbers] documentation.
    ///
    /// [Tokenized account numbers]: https://plaid.com/docs/auth/coverage/microdeposit-events/#tokenized-account-numbers
    pub account: String,

    /// Indicates whether the account number is tokenized by the institution.
    #[serde(default)]
    pub is_tokenized_account_number: Option<bool>,

    /// The ACH routing number for the account
    pub routing: String,

    /// The wire transfer routing number for the account.
    ///
    /// This field is only populated if the institution is known to use a
    /// different routing number for wire transfers.
    pub wire_routing: Option<String>,

    /// Whether the account supports ACH transfers into the account.
    #[serde(default)]
    pub can_transfer_in: Option<bool>,

    /// Whether the account supports ACH transfers out of the account.
    #[serde(default)]
    pub can_transfer_out: Option<bool>,
}

/// The numbers identifying an EFT account.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EftAccountNumbers {
    /// The Plaid account ID associated with the account numbers
    pub account_id: String,

    /// The EFT account number for the account
    pub account: String,

    /// The EFT institution number for the account
    pub institution: String,

    /// The EFT branch number for the account
    pub branch: String,
}

/// The numbers identifying an IBAN account.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InternationalAccountNumbers {
    /// The Plaid account ID associated with the account numbers
    pub account_id: String,

    /// The International Bank Account Number (IBAN) for the account
    pub iban: String,

    /// The Business Identifier Code (BIC) for the account
    pub bic: String,
}

/// The numbers identifying a BACS account.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BacsAccountNumbers {
    /// The Plaid account ID associated with the account numbers
    pub account_id: String,

    /// The BACS account number for the account
    pub account: String,

    /// The BACS sort code for the account
    pub sort_code: String,
}
