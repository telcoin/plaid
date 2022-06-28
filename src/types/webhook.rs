//! Webhooks

use serde::{Deserialize, Serialize};

/// The response from performing an `update_webhook` request
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebhookUpdateResponse {
    item: super::Item,
    request_id: String,
}

/// A broad categorization of the error. Safe for programmatic use.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookErrorType {
    /// Invalid Request Error
    InvalidRequest,
    /// Invalid Result Error
    InvalidResult,
    /// Invalid Input Error
    InvalidInput,
    /// Institution Error
    InstitutionError,
    /// Rate Limit Error
    RateLimitExceeded,
    /// API Error
    ApiError,
    /// Item Error
    ItemError,
    /// Asset Report Error
    AssetReportError,
    /// Recaptcha Error
    RecaptchaError,
    /// OAuth Error
    OauthError,
    /// Payment Error
    PaymentError,
    /// Bank Transfer Error
    BankTransferError,
    /// Income Verification Error
    IncomeVerificationError,
}

/// We use standard HTTP response codes for success and failure notifications, and our errors are
/// further classified by error_type. In general, 200 HTTP codes correspond to success, 40X codes
/// are for developer- or user-related failures, and 50X codes are for Plaid-related issues.
/// Error fields will be null if no error has occurred.
#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookError {
    /// A user-friendly representation of the error code. `null` if the error is not related to user
    /// action. This may change over time and is not safe for programmatic use.
    pub display_message: Option<String>,
    /// The particular error code
    pub error_code: String,
    /// A developer-friendly representation of the error code. This may change over time and is not
    /// safe for programmatic use.
    pub error_message: String,
    /// A broad categorization of the error
    pub error_type: WebhookErrorType,
    /// A unique ID identifying the request, to be used for troubleshooting purposes. This field
    /// will be omitted in errors provided by webhooks.
    pub request_id: Option<String>,
    /// In the `Assets` product, a request can pertain to more than one `Item`. If an error is returned
    /// for such a request, causes will return an array of errors containing a breakdown of these
    /// errors on the individual `Item` level, if any can be identified.
    /// `causes` will only be provided for the error_type `ASSET_REPORT_ERROR`. `causes` will also not be
    /// populated inside an error nested within a warning object.
    pub causes: Option<Vec<String>>,
    /// The HTTP status code associated with the error. This will only be returned in the response
    /// body when the error information is provided via a webhook.
    pub status: i32,
    /// The URL of a Plaid documentation page with more information about the error
    pub documentation_url: Option<String>,
    /// Suggested steps for resolving the error
    pub suggested_action: Option<String>,
}

/// The type of webhook
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "webhook_type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookType {
    /// Webhook relating to `Item`
    Item {
        /// Content of the Webhook
        #[serde(flatten)]
        content: crate::ItemWebhook,
    },
}

/// Top level webhook struct
#[derive(Serialize, Deserialize, Debug)]
pub struct Webhook {
    /// The type of webhook
    #[serde(flatten)]
    pub webhook_type: WebhookType,
    /// We use standard `HTTP` response codes for success and failure notifications, and our errors are
    /// further classified by `error_type`. In general, `200 HTTP` codes correspond to success, `40X` codes
    /// are for developer- or user-related failures, and `50X` codes are for Plaid-related issues.
    /// Error fields will be `null` if no error has occurred.
    pub error: Option<WebhookError>,
}

/// Module containing features for verifying webhooks
///
/// Relies on the [`openssl`] crate, which requires OpenSSL be installed
///
/// Only available with `webhook-verification` feature
#[cfg(feature = "webhook-verification")]
pub mod verification {
    use std::{
        error::Error as StdError,
        fmt::{Display, Formatter, Result as FmtResult},
    };

    use base64::decode_config;
    use jsonwebtoken::{
        jwk::{AlgorithmParameters, EllipticCurve, EllipticCurveKeyParameters, Jwk as BaseJwk},
        Algorithm,
    };
    use openssl::{bn::BigNum, ec::EcGroup, sha::sha256};
    use serde::{Deserialize, Serialize};

    use crate::Error;

    /// The possible errors of webhook verification
    #[derive(Debug)]
    pub enum WebhookVerificationError {
        /// An error occurred somewhere in the Api call or during transport
        ApiError(Error),
        /// A necessary parameter is missing
        MissingParameter(String),
        /// The incorrect algorithm was provided
        IncorrectAlgorithm,
        /// A value could not be parsed correctly
        CouldNotParse,
        /// The webhook could not be validated
        CouldNotValidate,
        /// An error occured in OpenSSL
        Cryptography,
    }
    impl From<Error> for WebhookVerificationError {
        fn from(error: Error) -> Self {
            Self::ApiError(error)
        }
    }
    impl StdError for WebhookVerificationError {}

    impl Display for WebhookVerificationError {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            write!(f, "{:?}", self)
        }
    }

    /// Custom wrapper around a JWK
    ///
    /// Plaid includes `created_at` and `expired_at` fields to counter replay attacks
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Jwk {
        /// The inner JWK
        #[serde(flatten)]
        pub inner: BaseJwk,
        /// Unix timestamp of when the JWK was created
        pub created_at: u64,
        /// Unix timestamp of when the JWK expires
        pub expired_at: Option<u64>,
    }
    impl Jwk {
        pub(crate) fn is_expired(&self) -> Option<bool> {
            let now = jsonwebtoken::get_current_timestamp();
            self.expired_at.map(|expired_at| expired_at < now)
        }
    }

    /// Response to the `/webhook_verification/get` request
    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct WebhookVerificationResponse {
        /// The JWK (JSON web key)
        pub key: Jwk,
        /// ID of the unique request
        pub request_id: String,
    }

    /// The JWT claims that Plaid will provide
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Claims {
        /// Issued At Time
        pub iat: u64,
        /// SHA256 of the webhook body
        pub request_body_sha256: String,
    }

    pub(crate) fn string_to_big_num(val: &str) -> Result<BigNum, WebhookVerificationError> {
        let b64 = decode_config(val, base64::URL_SAFE_NO_PAD)
            .map_err(|_| WebhookVerificationError::CouldNotParse)?;
        Ok(BigNum::from_slice(&b64).map_err(|_| WebhookVerificationError::CouldNotParse)?)
    }

    pub(crate) fn extract_key_id(token: &str) -> Result<String, WebhookVerificationError> {
        let header = jsonwebtoken::decode_header(&token)
            .map_err(|_| WebhookVerificationError::CouldNotParse)?;

        if header.alg != Algorithm::ES256 {
            return Err(WebhookVerificationError::IncorrectAlgorithm);
        }

        let kid = if let Some(kid) = header.kid {
            kid
        } else {
            return Err(WebhookVerificationError::MissingParameter(
                "kid".to_string(),
            ));
        };

        Ok(kid)
    }

    pub(crate) fn verify_webhook(
        key: &Jwk,
        token: &str,
        webhook_bytes: &[u8],
    ) -> Result<bool, WebhookVerificationError> {
        let (x, y) = match key.inner.algorithm {
            AlgorithmParameters::EllipticCurve(EllipticCurveKeyParameters {
                curve: EllipticCurve::P256,
                ref x,
                ref y,
                ..
            }) => (x, y),
            // Wrong algorithm
            _ => {
                return Ok(false);
            }
        };

        let x = string_to_big_num(x)?;
        let y = string_to_big_num(y)?;

        let ec_group = EcGroup::from_curve_name(openssl::nid::Nid::X9_62_PRIME256V1)
            .map_err(|_| WebhookVerificationError::Cryptography)?;

        let openssl_key = openssl::ec::EcKey::from_public_key_affine_coordinates(&ec_group, &x, &y)
            .map_err(|_| WebhookVerificationError::Cryptography)?;
        let openssl_key_pem = openssl_key
            .public_key_to_pem()
            .map_err(|_| WebhookVerificationError::Cryptography)?;

        let key = jsonwebtoken::DecodingKey::from_ec_pem(&openssl_key_pem)
            .map_err(|_| WebhookVerificationError::CouldNotParse)?;
        let mut val = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::ES256);

        // Don't check for `exp` field
        val.required_spec_claims = Default::default();
        val.validate_exp = false;

        let token_data = jsonwebtoken::decode::<Claims>(&token, &key, &val)
            .map_err(|_| WebhookVerificationError::CouldNotValidate)?;

        // verify time was within 5 minutes
        let now = jsonwebtoken::get_current_timestamp();
        if now - (5 * 60) > token_data.claims.iat {
            return Ok(false);
        }

        let webhook_sha = sha256(&webhook_bytes);
        let expected_sha: [u8; 32] = hex::FromHex::from_hex(&token_data.claims.request_body_sha256)
            .map_err(|_| WebhookVerificationError::CouldNotParse)?;

        Ok(webhook_sha == expected_sha)
    }
}
