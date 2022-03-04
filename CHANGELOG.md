# Changelog

### [v0.5.0](https://github.com/telcoin/plaid/compare/v0.4.0...v0.5.0) (2021-08-20)

### Features

* upgrade dependencies (tokio now at 1.17)

### [v0.4.0](https://github.com/telcoin/plaid/compare/v0.3.0...v0.4.0) (2021-08-20)

### Features

* parse API errors so they can be handled (#4) 05e0c1b


## [v0.3.0](https://github.com/telcoin/plaid/compare/v0.2.0...v0.3.0) (2021-08-16)

### Features

* make create_link_token send correct request (#3) 770c0f6


## [v0.2.0](https://github.com/telcoin/plaid/compare/v0.1.0...v0.2.0) (2021-08-16)

### ⚠ BREAKING CHANGE

* rename `CreateLinkTokenRequestParameters` to`CreateLinkTokenRequest`

* rename `CreatePublicTokenResponse` to`SandboxCreatePublicTokenResponse`

* change `sandbox_create_public_token` parameters

### Features

* support creating processor tokens (#2) 55bccb4
* add sandbox token creation configuration 46eee0c
* derive Clone & Copy where applicable 8bff2ba
* make AccountNumbers fields public ff6d54f


## v0.1.0 (2020-10-22)

### Features

* initial commit f770c65


