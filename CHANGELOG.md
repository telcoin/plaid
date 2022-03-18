# Changelog

### [v0.6.0](https://github.com/telcoin/plaid/compare/v0.5.1...v0.6.0) (2022-03-17)


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

