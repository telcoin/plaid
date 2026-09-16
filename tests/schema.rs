//! Deserializes the response examples published in Plaid's OpenAPI spec
//! (`2020-09-14`) to check that this crate's types match the documented shapes.
//!
//! Fixtures live in `tests/data/` and are copied verbatim from
//! <https://github.com/plaid/plaid-openapi>.

use plaid::{
    AccountsResponse, AuthResponse, CreateLinkTokenResponse, InstitutionResponse,
    WebhookUpdateResponse,
};

fn load(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/data/{}.json",
        env!("CARGO_MANIFEST_DIR"),
        name
    ))
    .unwrap_or_else(|e| panic!("could not read fixture `{}`: {}", name, e))
}

fn parse<T: serde::de::DeserializeOwned>(name: &str) -> T {
    serde_json::from_str(&load(name))
        .unwrap_or_else(|e| panic!("could not deserialize `{}`: {}", name, e))
}

#[test]
fn accounts_get() {
    let response: AccountsResponse = parse("accounts_get");
    assert!(!response.accounts.is_empty());
    assert!(response
        .accounts
        .iter()
        .all(|a| a.ty.subtype_str().is_some()));
}

#[test]
fn accounts_balance_get() {
    let response: AccountsResponse = parse("accounts_balance_get");
    assert!(!response.accounts.is_empty());
}

#[test]
fn auth_get() {
    let response: AuthResponse = parse("auth_get");
    assert!(!response.numbers.ach.is_empty());
}

#[test]
fn identity_get() {
    let response: AccountsResponse = parse("identity_get");
    assert!(response.accounts.iter().any(|a| !a.owners.is_empty()));
}

#[test]
fn institutions_get_by_id() {
    let response: InstitutionResponse = parse("institutions_get_by_id");
    let status = response.institution.status.expect("status");
    assert!(status.investments_updates.is_some());
    assert!(!response.institution.dtc_numbers.is_empty());
}

#[test]
fn item_webhook_update() {
    let _: WebhookUpdateResponse = parse("item_webhook_update");
}

#[test]
fn link_token_create() {
    let _: CreateLinkTokenResponse = parse("link_token_create");
}

/// Every value the spec documents must map to a real variant, never to the
/// `Unknown` catch-all. This is what guards the `rename_all` rules: a subtly
/// wrong rename would silently change the value sent on the wire.
#[test]
fn every_documented_value_is_known() {
    macro_rules! check {
        ($ty:ty, $unknown:path, $values:expr) => {
            for value in $values {
                let json = format!("\"{}\"", value);
                let parsed: $ty = serde_json::from_str(&json)
                    .unwrap_or_else(|e| panic!("{} rejected `{}`: {}", stringify!($ty), value, e));
                assert!(
                    !matches!(parsed, $unknown(_)),
                    "{} did not recognise the documented value `{}`",
                    stringify!($ty),
                    value,
                );
                assert_eq!(
                    serde_json::to_string(&parsed).unwrap(),
                    json,
                    "{} did not round-trip `{}`",
                    stringify!($ty),
                    value,
                );
            }
        };
    }

    check!(
        plaid::SupportedProduct,
        plaid::SupportedProduct::Unknown,
        PRODUCTS
    );
    check!(
        plaid::SupportedProcessor,
        plaid::SupportedProcessor::Unknown,
        PROCESSORS
    );
    check!(
        plaid::SupportedCountry,
        plaid::SupportedCountry::Unknown,
        COUNTRIES
    );
    check!(
        plaid::PhoneNumberType,
        plaid::PhoneNumberType::Unknown,
        PHONE_NUMBER_TYPES
    );
    check!(
        plaid::DepositorySubtype,
        plaid::DepositorySubtype::Unknown,
        DEPOSITORY_SUBTYPES
    );
    check!(
        plaid::CreditSubtype,
        plaid::CreditSubtype::Unknown,
        CREDIT_SUBTYPES
    );
    check!(
        plaid::LoanSubtype,
        plaid::LoanSubtype::Unknown,
        LOAN_SUBTYPES
    );
    check!(
        plaid::InvestmentSubtype,
        plaid::InvestmentSubtype::Unknown,
        INVESTMENT_SUBTYPES
    );
    check!(
        plaid::BrokerageSubtype,
        plaid::BrokerageSubtype::Unknown,
        BROKERAGE_SUBTYPES
    );
    check!(
        plaid::OtherSubtype,
        plaid::OtherSubtype::Unknown,
        OTHER_SUBTYPES
    );
}

const PRODUCTS: &[&str] = &[
    "assets",
    "auth",
    "balance",
    "balance_plus",
    "beacon",
    "identity",
    "identity_match",
    "investments",
    "investments_auth",
    "liabilities",
    "payment_initiation",
    "identity_verification",
    "transactions",
    "credit_details",
    "income",
    "income_verification",
    "standing_orders",
    "transfer",
    "employment",
    "recurring_transactions",
    "transactions_refresh",
    "signal",
    "statements",
    "processor_payments",
    "processor_identity",
    "profile",
    "cra_base_report",
    "cra_income_insights",
    "cra_partner_insights",
    "cra_network_insights",
    "cra_cashflow_insights",
    "cra_monitoring",
    "cra_lend_score",
    "cra_plaid_credit_score",
    "cra_qualify",
    "cra_home_lending",
    "layer",
    "pay_by_bank",
    "protect_linked_bank",
    "protect_transactions",
];

const PROCESSORS: &[&str] = &[
    "dwolla",
    "galileo",
    "modern_treasury",
    "ocrolus",
    "vesta",
    "drivewealth",
    "vopay",
    "achq",
    "check",
    "checkbook",
    "circle",
    "sila_money",
    "rize",
    "svb_api",
    "unit",
    "wyre",
    "lithic",
    "alpaca",
    "astra",
    "moov",
    "treasury_prime",
    "marqeta",
    "checkout",
    "solid",
    "highnote",
    "gemini",
    "apex_clearing",
    "gusto",
    "adyen",
    "atomic",
    "i2c",
    "wepay",
    "riskified",
    "utb",
    "adp_roll",
    "fortress_trust",
    "bond",
    "bakkt",
    "teal",
    "zero_hash",
    "taba_pay",
    "knot",
    "sardine",
    "alloy",
    "finix",
    "nuvei",
    "layer",
    "boom",
    "paynote",
    "stake",
    "wedbush",
    "esusu",
    "ansa",
    "scribeup",
    "straddle",
    "loanpro",
    "bloom_credit",
    "sfox",
    "brale",
    "parafin",
    "cardless",
    "open_ledger",
    "valon",
    "gainbridge",
    "cardlytics",
    "pinwheel",
    "thread_bank",
    "array",
    "fiant",
    "oatfi",
    "curinos",
    "frame",
    "interchecks",
    "interchange",
    "atomicfi",
    "pay",
    "natural",
    "kanmon",
    "kick",
    "increase",
    "airwallex",
    "cybrid",
    "bizcap",
    "webull",
];

const COUNTRIES: &[&str] = &[
    "US", "GB", "ES", "NL", "FR", "IE", "CA", "DE", "IT", "PL", "DK", "NO", "SE", "EE", "LT", "LV",
    "PT", "BE", "AT", "FI",
];

const PHONE_NUMBER_TYPES: &[&str] = &["home", "work", "office", "mobile", "mobile1", "other"];

const DEPOSITORY_SUBTYPES: &[&str] = &[
    "cash management",
    "cd",
    "checking",
    "ebt",
    "hsa",
    "limited purpose checking",
    "money market",
    "paypal",
    "prepaid",
    "savings",
];

const CREDIT_SUBTYPES: &[&str] = &["charge card", "credit card", "paypal"];

const LOAN_SUBTYPES: &[&str] = &[
    "auto",
    "business",
    "commercial",
    "commercial line of credit",
    "construction",
    "consumer",
    "home equity",
    "home equity loan",
    "installment",
    "line of credit",
    "loan",
    "mortgage",
    "other",
    "overdraft",
    "student",
];

const INVESTMENT_SUBTYPES: &[&str] = &[
    "401a",
    "401k",
    "403B",
    "457b",
    "529",
    "brokerage",
    "cash isa",
    "crypto exchange",
    "education savings account",
    "fhsa",
    "fixed annuity",
    "gic",
    "health reimbursement arrangement",
    "hsa",
    "ira",
    "isa",
    "keogh",
    "lif",
    "life insurance",
    "line of credit",
    "lira",
    "lrif",
    "lrsp",
    "mutual fund",
    "non-custodial wallet",
    "non-taxable brokerage account",
    "other",
    "other annuity",
    "other insurance",
    "pension",
    "prediction market",
    "prif",
    "profit sharing plan",
    "qshr",
    "rdsp",
    "resp",
    "retirement",
    "rlif",
    "roth",
    "roth 401k",
    "roth 403B",
    "roth 457b",
    "roth pension",
    "roth profit sharing plan",
    "roth thrift savings plan",
    "rrif",
    "rrsp",
    "sarsep",
    "sep ira",
    "simple ira",
    "sipp",
    "stock plan",
    "tfsa",
    "thrift savings plan",
    "trust",
    "ugma",
    "utma",
    "variable annuity",
];

const BROKERAGE_SUBTYPES: &[&str] = &[
    "brokerage",
    "non-custodial wallet",
    "non-taxable brokerage account",
];

const OTHER_SUBTYPES: &[&str] = &["other", "payroll"];

/// Values Plaid adds after this release must not break deserialization.
#[test]
fn unknown_enum_values_round_trip() {
    use plaid::{SupportedCountry, SupportedProduct};

    let product: SupportedProduct = serde_json::from_str("\"a_brand_new_product\"").unwrap();
    assert_eq!(
        product,
        SupportedProduct::Unknown("a_brand_new_product".to_string())
    );
    assert_eq!(
        serde_json::to_string(&product).unwrap(),
        "\"a_brand_new_product\""
    );

    let country: SupportedCountry = serde_json::from_str("\"FI\"").unwrap();
    assert_eq!(country, SupportedCountry::FI);
}

/// `type` and `subtype` arrive as two fields but are modelled as one value,
/// with the subtype scoped to the type that owns it.
#[test]
fn account_type_pairs_with_its_own_subtype() {
    use plaid::{AccountType, CreditSubtype, DepositorySubtype};

    fn parse(json: serde_json::Value) -> AccountType {
        serde_json::from_value(json).unwrap()
    }
    fn emit(ty: &AccountType) -> serde_json::Value {
        serde_json::to_value(ty).unwrap()
    }

    let checking = parse(serde_json::json!({"type": "depository", "subtype": "checking"}));
    assert_eq!(
        checking,
        AccountType::Depository(Some(DepositorySubtype::Checking))
    );
    assert_eq!(checking.as_str(), "depository");
    assert_eq!(checking.subtype_str().as_deref(), Some("checking"));

    // subtypes whose wire value is not snake_case
    let money_market = parse(serde_json::json!({"type": "depository", "subtype": "money market"}));
    assert_eq!(
        money_market,
        AccountType::Depository(Some(DepositorySubtype::MoneyMarket))
    );
    assert_eq!(
        emit(&money_market),
        serde_json::json!({"type": "depository", "subtype": "money market"})
    );

    // the same wire value under two types resolves to each type's own variant
    let credit_paypal = parse(serde_json::json!({"type": "credit", "subtype": "paypal"}));
    let depository_paypal = parse(serde_json::json!({"type": "depository", "subtype": "paypal"}));
    assert_eq!(
        credit_paypal,
        AccountType::Credit(Some(CreditSubtype::Paypal))
    );
    assert_eq!(
        depository_paypal,
        AccountType::Depository(Some(DepositorySubtype::Paypal))
    );

    // a null subtype is not an error
    let no_subtype = parse(serde_json::json!({"type": "loan", "subtype": null}));
    assert_eq!(no_subtype, AccountType::Loan(None));
    assert_eq!(no_subtype.subtype_str(), None);

    // an unknown subtype is kept, scoped to its type
    let new_subtype = parse(serde_json::json!({"type": "credit", "subtype": "future card"}));
    assert_eq!(
        new_subtype,
        AccountType::Credit(Some(CreditSubtype::Unknown("future card".to_string())))
    );
    assert_eq!(
        emit(&new_subtype),
        serde_json::json!({"type": "credit", "subtype": "future card"})
    );

    // an unknown *type* is preserved verbatim rather than rejected
    let new_type = parse(serde_json::json!({"type": "escrow", "subtype": "impound"}));
    assert_eq!(
        new_type,
        AccountType::Unknown {
            ty: "escrow".to_string(),
            subtype: Some("impound".to_string()),
        }
    );
    assert_eq!(
        emit(&new_type),
        serde_json::json!({"type": "escrow", "subtype": "impound"})
    );
}

/// An `Item` error is a full [`plaid::ApiError`], not an opaque blob.
#[test]
fn item_error_is_typed() {
    use plaid::{ErrorType, Item};

    let item: Item = serde_json::from_value(serde_json::json!({
        "item_id": "eVBnVMp7zdTJLkRNr33Rs6zr7KNJqBFL9DrE6",
        "institution_id": "ins_109508",
        "webhook": "https://www.genericwebhookurl.com/webhook",
        "available_products": ["balance", "identity", "investments"],
        "billed_products": ["assets", "auth", "transactions"],
        "consent_expiration_time": null,
        "update_type": "background",
        "error": {
            "error_type": "ITEM_ERROR",
            "error_code": "ITEM_LOGIN_REQUIRED",
            "error_message": "the login details of this item have changed",
            "display_message": "The login details of this bank account have changed.",
        },
    }))
    .unwrap();

    let error = item.error.expect("error");
    assert_eq!(error.error_type, ErrorType::ItemError);
    assert_eq!(error.error_code, "ITEM_LOGIN_REQUIRED");
    assert_eq!(
        item.available_products.unwrap(),
        vec![
            plaid::SupportedProduct::Balance,
            plaid::SupportedProduct::Identity,
            plaid::SupportedProduct::Investments,
        ]
    );
}
