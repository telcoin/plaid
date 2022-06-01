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
#[cfg(feature = "webhook-verification")]
pub mod verification {
    use base64::decode_config;
    use jsonwebtoken::{
        jwk::{self, AlgorithmParameters, EllipticCurve, EllipticCurveKeyParameters, Jwk},
        Algorithm,
    };
    use openssl::{bn::BigNum, ec::EcGroup, sha::sha256};
    use serde::{Deserialize, Serialize};

    use crate::Error;

    #[derive(Serialize, Deserialize, Debug)]
    pub enum WebhookVerificationError {
        MissingParameter(String),
        IncorrectAlgorithm,
        CouldNotParse,
        CouldNotValidate,
        Cryptography,
    }

    /// Response to the `/webhook_verification/get` request
    #[derive(Serialize, Deserialize, Debug)]
    pub struct WebhookVerificationResponse {
        /// The JWK (JSON web key)
        pub key: jwk::Jwk,
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

    pub(crate) fn string_to_big_num(val: &str) -> Result<BigNum, Error> {
        let b64 = decode_config(val, base64::URL_SAFE_NO_PAD)
            .map_err(|_| Error::WebhookVerification(WebhookVerificationError::CouldNotParse))?;
        Ok(BigNum::from_slice(&b64)
            .map_err(|_| Error::WebhookVerification(WebhookVerificationError::CouldNotParse))?)
    }

    pub fn extract_key_id_and_token(webhook: &reqwest::Request) -> Result<(String, String), Error> {
        let token = webhook
            .headers()
            .get("plaid-verification")
            .ok_or(Error::WebhookVerification(
                WebhookVerificationError::MissingParameter("plaid-verification".to_owned()),
            ))?
            .to_str()
            .map_err(|_| {
                Error::WebhookVerification(WebhookVerificationError::MissingParameter(
                    "plaid-verification".to_owned(),
                ))
            })?
            .to_owned();

        let header = jsonwebtoken::decode_header(&token)
            .map_err(|_| Error::WebhookVerification(WebhookVerificationError::CouldNotParse))?;

        if header.alg != Algorithm::ES256 {
            return Err(Error::WebhookVerification(
                WebhookVerificationError::IncorrectAlgorithm,
            ));
        }

        let kid = if let Some(kid) = header.kid {
            kid
        } else {
            return Err(Error::WebhookVerification(
                WebhookVerificationError::MissingParameter("kid".to_string()),
            ));
        };

        Ok((kid, token))
    }

    pub fn verify_webhook(key: Jwk, token: String, webhook_bytes: &[u8]) -> Result<bool, Error> {
        let (x, y) = match key.algorithm {
            AlgorithmParameters::EllipticCurve(EllipticCurveKeyParameters {
                curve: EllipticCurve::P256,
                x,
                y,
                ..
            }) => (x, y),
            // Wrong algorithm
            _ => return Ok(false),
        };

        let x = string_to_big_num(&x)?;
        let y = string_to_big_num(&y)?;

        let ec_group = EcGroup::from_curve_name(openssl::nid::Nid::X9_62_PRIME256V1)
            .map_err(|_| Error::WebhookVerification(WebhookVerificationError::Cryptography))?;

        let openssl_key = openssl::ec::EcKey::from_public_key_affine_coordinates(&ec_group, &x, &y)
            .map_err(|_| Error::WebhookVerification(WebhookVerificationError::Cryptography))?;
        let openssl_key_pem = openssl_key
            .public_key_to_pem()
            .map_err(|_| Error::WebhookVerification(WebhookVerificationError::Cryptography))?;

        let key = jsonwebtoken::DecodingKey::from_ec_pem(&openssl_key_pem)
            .map_err(|_| Error::WebhookVerification(WebhookVerificationError::CouldNotParse))?;
        let mut val = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::ES256);

        // Don't check for exp
        val.required_spec_claims = Default::default();
        val.validate_exp = false;

        let token_data = jsonwebtoken::decode::<Claims>(&token, &key, &val)
            .map_err(|_| Error::WebhookVerification(WebhookVerificationError::CouldNotValidate))?;

        // verify time was within 5 minutes
        let now = jsonwebtoken::get_current_timestamp();
        if now - (5 * 60) > token_data.claims.iat {
            return Ok(false);
        }

        let webhook_sha = sha256(&webhook_bytes);
        let expected_sha: [u8; 32] = hex::FromHex::from_hex(&token_data.claims.request_body_sha256)
            .map_err(|_| Error::WebhookVerification(WebhookVerificationError::CouldNotParse))?;

        Ok(webhook_sha == expected_sha)
    }
}
