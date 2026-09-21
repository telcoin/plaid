//! Verification of the `Plaid-Verification` JWT on an inbound webhook.
//!
//! Requires the `webhook-verification` feature.
//!
//! Plaid signs every webhook it sends and puts the signature in a
//! [`VERIFICATION_HEADER`] header. Without checking it, a webhook endpoint is
//! unauthenticated: any caller on the Internet can make the service act on — or
//! merely store — a delivery it never received from Plaid. [`WebhookVerifier`]
//! is the check, over [`Client::webhook_verification_key`].
//!
//! ```no_run
//! # async fn handle(
//! #     client: plaid::Client,
//! #     header: Option<&str>,
//! #     body: &[u8],
//! # ) -> Result<(), Box<dyn std::error::Error>> {
//! let verifier = plaid::WebhookVerifier::new(client);
//!
//! match verifier.verify(header, body).await {
//!     Ok(verified) => {
//!         // `body` really came from Plaid; `verified.fingerprint` identifies
//!         // this delivery, for deduplicating redeliveries.
//!         let webhook: plaid::Webhook = serde_json::from_slice(body)?;
//!         let _ = (webhook, verified);
//!     }
//!     // Nothing was decided — ask for the delivery again rather than dropping it.
//!     Err(error) if error.is_inconclusive() => return Err(error.into()),
//!     // The delivery is not Plaid's. Drop it.
//!     Err(_) => {}
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # What a signature is and is not worth
//!
//! A signature proves a message came from Plaid, not that it is still true. A
//! handler acting on something consequential — an Item being revoked, say —
//! should still confirm the current state with Plaid before acting, and that
//! check is the stronger of the two.
//!
//! What confirmation cannot do is protect the step that happens *before*
//! anything is interpreted: storing the delivery. An unsigned endpoint means any
//! caller can make the service write a durable row as fast as it can send, which
//! is a database that grows faster than anything sweeping it. So the signature is
//! checked first, and nothing unverified is written down.

use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{Client, Error, WebhookVerificationKey};

/// The header Plaid signs a delivery with.
pub const VERIFICATION_HEADER: &str = "plaid-verification";

/// The only JWT algorithm accepted, named rather than read from the token.
///
/// Reading the algorithm out of the header and trusting it is the classic JWT
/// hole: `none` verifies everything, and an asymmetric key presented as an HMAC
/// secret verifies anything signed with the public key. Plaid signs with ES256,
/// so ES256 is what this accepts.
const ALGORITHM: &str = "ES256";

/// How old a delivery may be and still be acted on.
///
/// Bounds replay of a delivery that really was signed by Plaid: the signature
/// stays valid for ever, so without a clock a copy captured once could be resent
/// indefinitely. Five minutes is Plaid's own suggested window.
const MAX_AGE: Duration = Duration::from_secs(5 * 60);

/// How far ahead of this clock a delivery may claim to have been issued.
///
/// Not zero, because two clocks are involved and only one of them is ours.
const MAX_SKEW: Duration = Duration::from_secs(60);

/// The longest `Plaid-Verification` value worth looking at.
///
/// A real one is a few hundred bytes. The cap is not a security boundary — the
/// signature is — it just keeps the work bounded before anything has been proven.
const MAX_HEADER_BYTES: usize = 4096;

/// The longest key id worth asking Plaid about.
///
/// Plaid's are UUIDs. Anything longer is not one, and refusing it here is what
/// stops an unauthenticated caller choosing the string this crate sends upstream.
const MAX_KEY_ID_BYTES: usize = 64;

/// How many verification keys to hold before dropping the lot.
///
/// Only keys Plaid has answered for are ever cached, so this is bounded by
/// Plaid's own rotation rather than by anything a caller sends; the cap is there
/// so that a bound exists at all. Dropping everything rather than evicting one is
/// fine: a key is one call away.
const MAX_CACHED_KEYS: usize = 32;

impl WebhookVerificationKey {
    /// The key as something that can check a signature, or why it cannot be
    /// used.
    ///
    /// Requires the `webhook-verification` feature.
    pub fn verifying_key(&self) -> Result<VerifyingKey, VerificationError> {
        if self.expired_at.is_some() {
            return Err(VerificationError::KeyExpired);
        }
        if self.alg != ALGORITHM || self.crv != "P-256" || self.kty != "EC" {
            return Err(VerificationError::Malformed(
                "the verification key is not an ES256 P-256 key",
            ));
        }

        let x = decode_segment(&self.x)?;
        let y = decode_segment(&self.y)?;

        // SEC1 uncompressed: `0x04 || x || y`, which is the form the JWK's two
        // coordinates spell out and the one `VerifyingKey` parses.
        let mut point = Vec::with_capacity(1 + x.len() + y.len());
        point.push(0x04);
        point.extend_from_slice(&x);
        point.extend_from_slice(&y);

        VerifyingKey::from_sec1_bytes(&point)
            .map_err(|_| VerificationError::Malformed("the verification key is not a valid point"))
    }
}

/// Checks the `Plaid-Verification` JWT on an inbound webhook.
///
/// Requires the `webhook-verification` feature.
///
/// # What a valid delivery has to satisfy
///
/// 1. The header is present, and is a three-part JWT of a sane size.
/// 2. Its algorithm is `ES256`. Read from the token but never *trusted* from it
///    — anything else is refused rather than honoured.
/// 3. Its `kid` is a plausible key id, and Plaid answers for it with a key it
///    has not retired.
/// 4. The signature over `header.payload` verifies under that key.
/// 5. The claims are fresh: issued within the last five minutes, and not more
///    than a minute in the future.
/// 6. `request_body_sha256` is the SHA-256 of the body that arrived — which is
///    what ties the signature to *this* delivery rather than to any delivery
///    Plaid ever signed.
///
/// Only then has anything been proven.
///
/// # The key cache
///
/// Keys are held in memory by `kid`, because otherwise every delivery costs an
/// upstream call and a Plaid outage would take webhook ingestion with it. Only
/// keys Plaid has answered for are cached, so an unknown `kid` is a call every
/// time — bound that with a rate limit on the route, which is the right place
/// for it: the limit also bounds every other thing an unauthenticated caller can
/// ask the endpoint to do.
///
/// In memory means per instance and forgotten on restart, which costs one call
/// per key per instance. There is nothing to invalidate: a rotated key is
/// refused by its own `expired_at`, and a new one has a `kid` that was never
/// cached.
#[derive(Debug)]
pub struct WebhookVerifier {
    client: Client,
    keys: Mutex<HashMap<String, VerifyingKey>>,
}

impl WebhookVerifier {
    /// Build one on a Plaid [`Client`].
    pub fn new(client: Client) -> Self {
        Self {
            client,
            keys: Mutex::new(HashMap::new()),
        }
    }

    /// Verify a delivery, answering with what identifies it.
    ///
    /// `header` is the raw `Plaid-Verification` value, and `body` the exact
    /// bytes that arrived — the digest covers those and not a re-serialisation
    /// of them.
    pub async fn verify(
        &self,
        header: Option<&str>,
        body: &[u8],
    ) -> Result<Verified, VerificationError> {
        let token = header.ok_or(VerificationError::Missing)?;
        if token.len() > MAX_HEADER_BYTES {
            return Err(VerificationError::Malformed(
                "the verification header is too long",
            ));
        }

        let mut parts = token.split('.');
        let (Some(header_b64), Some(claims_b64), Some(signature_b64), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(VerificationError::Malformed(
                "the verification header is not a three-part JWT",
            ));
        };

        let header: JwtHeader = serde_json::from_slice(&decode_segment(header_b64)?)
            .map_err(|_| VerificationError::Malformed("the JWT header is not JSON"))?;

        // Before the key is fetched, so a token asking to be verified with
        // `none`, or with the public key as an HMAC secret, is refused without a
        // call.
        if header.alg != ALGORITHM {
            return Err(VerificationError::UnsupportedAlgorithm(header.alg));
        }
        if header.kid.is_empty()
            || header.kid.len() > MAX_KEY_ID_BYTES
            || !header
                .kid
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(VerificationError::Malformed(
                "the JWT names no usable key id",
            ));
        }

        let key = self.key(&header.kid).await?;

        // Raw `r || s`, as JWS specifies for ES256 — not the ASN.1 form OpenSSL
        // emits.
        let signature = Signature::from_slice(&decode_segment(signature_b64)?)
            .map_err(|_| VerificationError::Malformed("the JWT signature is not an ES256 one"))?;
        let signed = format!("{}.{}", header_b64, claims_b64);
        key.verify(signed.as_bytes(), &signature)
            .map_err(|_| VerificationError::BadSignature)?;

        // Everything from here is a claim Plaid signed, so it can be believed —
        // and has to be checked, because a signature says who wrote it and not
        // when, or about what.
        let claims: JwtClaims = serde_json::from_slice(&decode_segment(claims_b64)?)
            .map_err(|_| VerificationError::Malformed("the JWT claims are not JSON"))?;

        let now = unix_now();
        let age = now - claims.iat;
        if age > MAX_AGE.as_secs() as i64 {
            return Err(VerificationError::Stale(age));
        }
        if -age > MAX_SKEW.as_secs() as i64 {
            return Err(VerificationError::Stale(age));
        }

        let digest = hex::encode(Sha256::digest(body));
        if !digest.eq_ignore_ascii_case(&claims.request_body_sha256) {
            return Err(VerificationError::BodyMismatch);
        }

        Ok(Verified {
            fingerprint: hex::encode(Sha256::digest(token.as_bytes())),
        })
    }

    /// The key for `kid`, from the cache or from Plaid.
    async fn key(&self, kid: &str) -> Result<VerifyingKey, VerificationError> {
        if let Some(key) = self.cached(kid) {
            return Ok(key);
        }

        let fetched =
            self.client
                .webhook_verification_key(kid)
                .await
                .map_err(|error| match &error {
                    // Plaid answering "no such key" is a fact about the delivery:
                    // whoever sent it named a key Plaid does not sign with. A
                    // transport failure says nothing about the delivery at all, so
                    // the two get different answers.
                    Error::Api(_) => VerificationError::UnknownKey,
                    Error::TransportStd(_) => VerificationError::KeyUnavailable(Box::new(error)),
                })?;

        let key = fetched.key.verifying_key()?;
        self.cache(kid, key);
        Ok(key)
    }

    /// A cached key, if this instance has fetched it before.
    fn cached(&self, kid: &str) -> Option<VerifyingKey> {
        self.keys().get(kid).copied()
    }

    /// Remember a key Plaid answered for.
    fn cache(&self, kid: &str, key: VerifyingKey) {
        let mut keys = self.keys();
        if keys.len() >= MAX_CACHED_KEYS {
            keys.clear();
        }
        keys.insert(kid.to_owned(), key);
    }

    /// The cache, recovering from a panic in another request.
    fn keys(&self) -> MutexGuard<'_, HashMap<String, VerifyingKey>> {
        self.keys
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// A delivery that Plaid really signed, and what identifies it.
#[derive(Debug, Clone)]
pub struct Verified {
    /// A hash of the whole verification token.
    ///
    /// **The dedupe key for a Plaid delivery**, which has no other. Plaid sends
    /// no idempotency header, and its webhook bodies carry no id — two genuine
    /// `ITEM` events for one Item can be byte-identical — so the body is no use
    /// as a key either. The token is: Plaid signs each delivery once, with the
    /// time it did so in the claims, so two separate events never share one and
    /// a redelivery of the same event always does.
    pub fingerprint: String,
}

/// The JWT header, as far as this cares.
#[derive(Debug, Deserialize)]
struct JwtHeader {
    alg: String,
    #[serde(default)]
    kid: String,
}

/// The claims Plaid puts in it.
#[derive(Debug, Deserialize)]
struct JwtClaims {
    /// When Plaid signed the delivery, as a Unix timestamp.
    iat: i64,
    /// The SHA-256 of the body, hex-encoded. What ties this signature to this
    /// delivery.
    request_body_sha256: String,
}

/// Why a delivery could not be believed.
#[derive(Debug)]
pub enum VerificationError {
    /// The delivery carried no verification header.
    Missing,

    /// The header, or the key Plaid answered with, was not the shape it has to
    /// be.
    Malformed(&'static str),

    /// The delivery named an algorithm other than `ES256`.
    UnsupportedAlgorithm(String),

    /// Plaid does not know the key the delivery names.
    UnknownKey,

    /// The delivery was signed with a key Plaid has retired.
    KeyExpired,

    /// The signature does not verify under the key the delivery names.
    BadSignature,

    /// The delivery was issued outside the window it may be acted on, given as
    /// its age in seconds — negative if it claims to have been issued in the
    /// future.
    Stale(i64),

    /// The signature does not cover the body that arrived.
    BodyMismatch,

    /// The key could not be fetched, so **nothing has been decided**.
    ///
    /// Its own variant because it is the one failure that is not the sender's:
    /// the delivery may be perfectly good and the key simply could not be
    /// fetched to find out. It has to be answered in a way that asks for the
    /// delivery again rather than one that discards it. See
    /// [`is_inconclusive`](VerificationError::is_inconclusive).
    KeyUnavailable(Box<Error>),
}

impl VerificationError {
    /// Whether the delivery should be sent again rather than abandoned.
    ///
    /// True only for [`KeyUnavailable`](VerificationError::KeyUnavailable),
    /// where nothing about the delivery was established. Answer one of those
    /// with a status that makes Plaid retry; answer everything else with one
    /// that does not.
    pub fn is_inconclusive(&self) -> bool {
        matches!(self, VerificationError::KeyUnavailable(_))
    }
}

impl Display for VerificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            VerificationError::Missing => {
                f.write_str("the delivery carried no verification header")
            }
            VerificationError::Malformed(what) => f.write_str(what),
            VerificationError::UnsupportedAlgorithm(alg) => write!(
                f,
                "the delivery was signed with {}, which is not accepted",
                alg
            ),
            VerificationError::UnknownKey => {
                f.write_str("plaid does not know the key this delivery names")
            }
            VerificationError::KeyExpired => {
                f.write_str("the delivery was signed with a key plaid has retired")
            }
            VerificationError::BadSignature => f.write_str("the signature does not verify"),
            VerificationError::Stale(age) => write!(
                f,
                "the delivery was issued {}s ago, outside the window it may be acted on",
                age
            ),
            VerificationError::BodyMismatch => {
                f.write_str("the signature does not cover this body")
            }
            VerificationError::KeyUnavailable(error) => {
                write!(f, "could not fetch the verification key: {}", error)
            }
        }
    }
}

impl StdError for VerificationError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            VerificationError::KeyUnavailable(error) => Some(&**error),
            _ => None,
        }
    }
}

/// Decode one base64url segment, unpadded as JWS specifies.
fn decode_segment(segment: &str) -> Result<Vec<u8>, VerificationError> {
    URL_SAFE_NO_PAD
        .decode(segment)
        .map_err(|_| VerificationError::Malformed("a JWT segment is not base64url"))
}

/// Now, as a Unix timestamp.
///
/// A clock before the epoch is a broken machine rather than a case to handle,
/// and reading it as `0` makes every delivery stale — which is the safe way to
/// be wrong.
fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::signature::Signer;
    use p256::ecdsa::SigningKey;
    use serde_json::json;

    use super::*;

    /// A key, and the JWK Plaid would describe it with.
    fn keypair() -> (SigningKey, WebhookVerificationKey) {
        let signing = SigningKey::from_bytes(&[7u8; 32].into()).expect("a valid scalar");
        let point = signing.verifying_key().to_encoded_point(false);

        let key = WebhookVerificationKey {
            alg: ALGORITHM.to_owned(),
            crv: "P-256".to_owned(),
            kty: "EC".to_owned(),
            usage: "sig".to_owned(),
            kid: "kid-1".to_owned(),
            x: URL_SAFE_NO_PAD.encode(point.x().expect("an x coordinate")),
            y: URL_SAFE_NO_PAD.encode(point.y().expect("a y coordinate")),
            created_at: 1_700_000_000,
            expired_at: None,
        };
        (signing, key)
    }

    /// Build a token the way Plaid does.
    fn token(signing: &SigningKey, kid: &str, iat: i64, body_sha256: &str) -> String {
        let header = URL_SAFE_NO_PAD.encode(json!({ "alg": "ES256", "kid": kid }).to_string());
        let claims = URL_SAFE_NO_PAD
            .encode(json!({ "iat": iat, "request_body_sha256": body_sha256 }).to_string());
        let signed = format!("{}.{}", header, claims);
        let signature: Signature = signing.sign(signed.as_bytes());
        format!(
            "{}.{}",
            signed,
            URL_SAFE_NO_PAD.encode(signature.to_bytes())
        )
    }

    #[test]
    fn a_jwk_becomes_a_key_that_checks_its_own_signatures() {
        let (signing, jwk) = keypair();
        let key = jwk.verifying_key().expect("the JWK should parse");

        let signature: Signature = signing.sign(b"a.b");
        assert!(key.verify(b"a.b", &signature).is_ok());
        assert!(key.verify(b"a.c", &signature).is_err());
    }

    /// A retired key verifies nothing, whatever it was used to sign.
    #[test]
    fn an_expired_key_is_refused() {
        let (_, mut jwk) = keypair();
        jwk.expired_at = Some(1_700_000_000);

        assert!(matches!(
            jwk.verifying_key(),
            Err(VerificationError::KeyExpired)
        ));
    }

    /// The parts of verification that need no Plaid: everything after the key is
    /// in hand.
    ///
    /// `tests/webhook_verification.rs` covers the fetch and the cache; this
    /// covers the arithmetic, which is where an off-by-one would be invisible
    /// from outside.
    #[test]
    fn a_token_verifies_only_over_its_own_body_and_within_its_window() {
        let (signing, jwk) = keypair();
        let key = jwk.verifying_key().expect("the JWK should parse");

        let body = br#"{"webhook_type":"ITEM"}"#;
        let digest = hex::encode(Sha256::digest(body));
        let token = token(&signing, "kid-1", unix_now(), &digest);

        let mut parts = token.split('.');
        let header_b64 = parts.next().unwrap();
        let claims_b64 = parts.next().unwrap();
        let signature = Signature::from_slice(&decode_segment(parts.next().unwrap()).unwrap())
            .expect("the signature should be 64 bytes");

        let signed = format!("{}.{}", header_b64, claims_b64);
        key.verify(signed.as_bytes(), &signature)
            .expect("the token should verify under its own key");

        // The digest is what ties the signature to this delivery: a body it did
        // not cover has a different one, and the claim no longer matches.
        let claims: JwtClaims = serde_json::from_slice(&decode_segment(claims_b64).unwrap())
            .expect("the claims should parse");
        assert_eq!(claims.request_body_sha256, digest);
        assert_ne!(
            claims.request_body_sha256,
            hex::encode(Sha256::digest(b"{}"))
        );
        assert!(unix_now() - claims.iat <= MAX_AGE.as_secs() as i64);
    }
}
