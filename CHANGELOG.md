# Changelog

## [Unreleased]

### Features

* add `Client::remove_item` for [/item/remove], the only call that ends an Item
  at Plaid. Until it succeeds, Plaid keeps serving — and billing for — a
  credential the user may already have asked you to give up
* add `Error::is_item_not_found`, which reads Plaid's `ITEM_NOT_FOUND` as "the
  state you asked for holds". `/item/remove` answers with it for an Item that is
  already gone, so a retry after a partial failure is safe
* add `Client::webhook_verification_key` for [/webhook_verification_key/get],
  along with `WebhookVerificationKeyResponse` and `WebhookVerificationKey`
* add a `webhook-verification` feature, off by default, providing
  `WebhookVerifier` — the check on the `Plaid-Verification` JWT that an inbound
  webhook carries. Without it a webhook endpoint is unauthenticated: any caller
  can make the service store, or act on, a delivery Plaid never sent. It bounds
  what the sender chooses before asking Plaid about it, refuses any algorithm but
  ES256 and any key Plaid has retired, ties the signature to the body that
  arrived, bounds replay with Plaid's five-minute window, caches keys by `kid`,
  and distinguishes a delivery it refused from one it could not decide about
  (`VerificationError::is_inconclusive`)

[/item/remove]: https://plaid.com/docs/api/items/#itemremove
[/webhook_verification_key/get]: https://plaid.com/docs/api/webhooks/webhook-verification/#webhook_verification_keyget

### [v0.12.0](https://github.com/telcoin/plaid/compare/v0.9.1...v0.12.0) (2026-09-16)

Brings the crate in line with the [Plaid API docs](https://plaid.com/docs/api/)
as of the `2020-09-14_1.729.1` OpenAPI spec.

### ⚠ BREAKING CHANGES

* `Item::error` is now an `Option<ApiError>` rather than an `Option<serde_json::Value>`
* `Item::available_products` / `billed_products` are now `Option<Vec<SupportedProduct>>` rather than `Option<Vec<String>>`
* `Institution::products` and `Institution::country_codes` are now `Vec<SupportedProduct>` and `Vec<SupportedCountry>` rather than `Vec<String>`
* `InstitutionStatus` fields are now `Option<RequestStatus>`; Plaid returns each product's status only when it has enough traffic to compute it
* `InstitutionStatus::investment_update` is now `investments_updates`, which is the name the API actually uses — the old field never deserialized
* `Breakdown::refresh_interval`, `HealthIncident::end_date` and the `IncidentUpdate` fields are now optional, matching the spec
* `Account::ty` and `Account::subtype` are collapsed into a single flattened `ty: AccountType`. `AccountType` is now a data-carrying enum pairing each type with a subtype enum scoped to it — `AccountType::Depository(Option<DepositorySubtype>)`, `Credit(Option<CreditSubtype>)`, `Loan(..)`, `Investment(..)`, `Brokerage(..)`, `Other(..)` — so a subtype cannot be paired with a type it does not belong to. An unrecognised type is preserved as `AccountType::Unknown { ty, subtype }` rather than rejected
* the flat `AccountSubtype` enum is replaced by the per-type `DepositorySubtype`, `CreditSubtype`, `LoanSubtype`, `InvestmentSubtype`, `BrokerageSubtype` and `OtherSubtype`
* `AccountFilters` fields are now `AccountSubtypeFilter<T>` over the matching per-type subtype enum, and the wildcard is `SubtypeSelector::All`
* `PhoneNumberType::Other` is now a unit variant for the documented `other` value; unrecognised values land in the new `PhoneNumberType::Unknown(String)`
* `SupportedProduct`, `SupportedCountry` and `SupportedProcessor` are no longer `Copy`; each gained an `Unknown(String)` variant so values Plaid adds later round-trip instead of failing to deserialize
* `SupportedProcessor` no longer has `InteractiveBrokers` or `PrimeTrust`; Plaid no longer documents either as a valid processor
* `CreateLinkTokenRequest::account_filters` is now a typed `AccountFilters` rather than a `serde_json::Map`
* `PaymentInitiationConfiguration::payment_id` is now optional, and `consent_id` was added
* `BalanceRequestOptions::min_last_updated_datetime` is now a `DateTime<FixedOffset>` rather than a `String`
* `Client::institution_by_id` now takes `&[SupportedCountry]` rather than `&[&str]`
* `EndUser` and `CreateLinkTokenRequest` gained fields; both now implement `Default`, so construct them with `..Default::default()`
* `Environment` is no longer `Copy`, because `Environment::Custom` owns a `String`; it is still `Clone`

### Features

* add `Environment::Custom`, which points the client at an arbitrary base URL such as a mock server or proxy, along with `Environment::base_url()`. `Environment`'s `FromStr` now accepts an `http://` or `https://` URL, so `PLAID_ENVIRONMENT` can select one
* add `Item::institution_name`, `products`, `consented_products`, `update_type` and `auth_method`
* add `Account::verification_name`, `persistent_account_id`, `apy` and `holder_category`
* add `AccountType::Brokerage`, plus `AccountType::as_str`, `subtype_str` and a `Display` impl
* add `Balances::last_updated_datetime`
* add the remaining `VerificationStatus` variants (`unsent`, `verification_failed`, `database_matched`, and the Database Insights statuses)
* add `AchAccountNumbers::is_tokenized_account_number`, `can_transfer_in` and `can_transfer_out`
* add `CreateLinkTokenRequest::required_if_supported_products`, `optional_products`, `additional_consented_products`, `hosted_link` and `user_token`
* add `CreateLinkTokenResponse::hosted_link_url`, `user_id` and `request_id`
* add `Institution::dtc_numbers`, `PaymentInitiationMetadata::supports_payment_consents` and `SupportedMethods::instant_micro_deposits`
* add `ApiError::error_code_reason`, `status`, `causes`, `required_account_subtypes` and `provided_account_subtypes`
* add the `LOGIN_REPAIRED`, `PENDING_DISCONNECT` and `USER_ACCOUNT_REVOKED` Item webhook codes, plus `user_id` and `environment` on `ItemWebhook`
* expand `SupportedProduct`, `SupportedCountry`, `SupportedProcessor`, `SupportedLanguage`, `ErrorType` and `WebhookErrorType` to the currently documented values
* make `WebhookUpdateResponse::item` and `request_id` public — they were private, so the response could not be read
* add `tests/schema.rs`, which deserializes the response examples published in Plaid's OpenAPI spec and asserts that every string value the spec documents maps to a real variant and round-trips unchanged

### Internal

* replace the hand-written `wire_enum!` and per-string `named_unit_variant!` macros with [`serde-enum-str`](https://crates.io/crates/serde-enum-str) derives; the string enums are now plain `#[serde(rename_all = ...)]` / `#[serde(other)]` declarations. Adds six build-time proc-macro dependencies and no runtime ones

### Fixes

* deprecate `Environment::Development`; Plaid has retired the Development environment and only lists Sandbox and Production as API hosts
* send `options` (not `option`) in `/institutions/get_by_id` requests, so the request options were previously ignored
* deprecate `ErrorType::DepositSwitchError`; Plaid has retired the Deposit Switch product
* point doc links at the current API reference pages

### [v0.9.1](https://github.com/telcoin/plaid/compare/v0.9.0...v0.9.1) (2022-06-17)


### Fixes

* make Balance::current nullable (#9)
 db3e13e


## [v0.9.0](https://github.com/telcoin/plaid/compare/v0.8.0...v0.9.0) (2022-05-27)


### Features

* add min datetime to balance request
 6969eda


## [v0.8.0](https://github.com/telcoin/plaid/compare/v0.7.0...v0.8.0) (2022-05-27)


### Features

* add webhooks with std async/await (#7)
 c3a6a7b


## [v0.7.0](https://github.com/telcoin/plaid/compare/v0.6.0...v0.7.0) (2022-05-06)


### Features

* ensure all types are serialize & deserialize
 a22268a


## [v0.6.0](https://github.com/telcoin/plaid/compare/v0.5.1...v0.6.0) (2022-03-18)


### Features

* derive Clone and Debug for Client
 95eef4d


### [v0.5.1](https://github.com/telcoin/plaid/compare/v0.5.0...v0.5.1) (2022-03-18)


## [v0.5.0](https://github.com/telcoin/plaid/compare/v0.4.0...v0.5.0) (2022-03-18)

### ⚠ BREAKING CHANGE

* futures 0.1 is no longer supported and the cargo feature has been removed


### Features

* remove support for futures 0.1
 7ac58bf


## [v0.4.0](https://github.com/telcoin/plaid/compare/v0.3.0...v0.4.0) (2021-08-20)


### Features

* parse API errors so they can be handled (#4)
 05e0c1b


## [v0.3.0](https://github.com/telcoin/plaid/compare/v0.2.0...v0.3.0) (2021-08-16)


### Features

* make create_link_token send correct request (#3)
 770c0f6


## [v0.2.0](https://github.com/telcoin/plaid/compare/v0.1.0...v0.2.0) (2021-08-16)

### ⚠ BREAKING CHANGE

* rename `CreateLinkTokenRequestParameters` to`CreateLinkTokenRequest`
* rename `CreatePublicTokenResponse` to`SandboxCreatePublicTokenResponse`
* change `sandbox_create_public_token` parameters


### Features

* support creating processor tokens (#2)
 55bccb4

* add sandbox token creation configuration
 46eee0c

* derive Clone & Copy where applicable
 8bff2ba

* make AccountNumbers fields public
 ff6d54f


## v0.1.0 (2020-10-22)


### Features

* initial commit
 f770c65

