//! Checks [`plaid::WebhookVerifier`] end to end, against a local listener
//! standing in for `/webhook_verification_key/get`.
//!
//! The unit tests in `src/verification.rs` cover the arithmetic once a key is in
//! hand. What only shows up from out here is the rest: that the key is fetched
//! over the real client, that it is fetched *once*, and that the checks which
//! are meant to happen before the fetch really do.

#![cfg(feature = "webhook-verification")]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::thread::JoinHandle;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use plaid::{Client, Environment, VerificationError, WebhookVerifier};
use serde_json::json;
use sha2::{Digest, Sha256};

/// A listener that answers exactly `responses.len()` requests and then stops.
///
/// Exact rather than open-ended so that an unexpected extra call fails the test
/// by being refused, rather than passing quietly. Each response closes its
/// connection, so one request is one connection and the count is the count.
fn serve(responses: Vec<(u16, String)>) -> (SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("local addr");

    let handle = std::thread::spawn(move || {
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));

            let mut line = String::new();
            reader.read_line(&mut line).expect("request line");

            let mut content_length = 0;
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).expect("header");
                if header.trim().is_empty() {
                    break;
                }
                if let Some(value) = header.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_length = value.trim().parse().expect("content-length");
                }
            }
            let mut request_body = vec![0; content_length];
            reader.read_exact(&mut request_body).expect("body");

            write!(
                stream,
                "HTTP/1.1 {} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status,
                body.len(),
                body,
            )
            .expect("respond");
            stream.flush().expect("flush");
        }
    });

    (address, handle)
}

/// A client pointed at the given address, or at nothing in particular.
fn client(address: SocketAddr) -> Client {
    Client::new(
        "test_client_id",
        "test_secret".to_string(),
        Environment::Custom(format!("http://{}", address)),
    )
}

/// The key the tests sign with, and Plaid's answer for it.
fn keypair(kid: &str) -> (SigningKey, String) {
    let signing = SigningKey::from_bytes(&[7u8; 32].into()).expect("a valid scalar");
    let point = signing.verifying_key().to_encoded_point(false);

    let response = json!({
        "key": {
            "alg": "ES256",
            "crv": "P-256",
            "kty": "EC",
            "use": "sig",
            "kid": kid,
            "x": URL_SAFE_NO_PAD.encode(point.x().expect("an x coordinate")),
            "y": URL_SAFE_NO_PAD.encode(point.y().expect("a y coordinate")),
            "created_at": 1_560_466_150,
            "expired_at": serde_json::Value::Null,
        },
        "request_id": "test-request-id",
    })
    .to_string();

    (signing, response)
}

/// Build a token the way Plaid does: ES256 over `header.claims`, with the body's
/// digest in the claims.
fn token(signing: &SigningKey, kid: &str, iat: i64, body: &[u8]) -> String {
    let header = URL_SAFE_NO_PAD.encode(json!({ "alg": "ES256", "kid": kid }).to_string());
    let claims = URL_SAFE_NO_PAD.encode(
        json!({
            "iat": iat,
            "request_body_sha256": hex::encode(Sha256::digest(body)),
        })
        .to_string(),
    );
    let signed = format!("{}.{}", header, claims);
    let signature: Signature = signing.sign(signed.as_bytes());
    format!(
        "{}.{}",
        signed,
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    )
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("a clock after the epoch")
        .as_secs() as i64
}

/// An address nothing is listening on, for the cases where a call must *not* be
/// made — or must fail.
fn nowhere() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("local addr");
    drop(listener);
    address
}

/// A genuine delivery verifies, and the second one costs no upstream call.
///
/// The listener answers exactly once, so if the cache did not hold the key the
/// second `verify` would be refused a connection and come back inconclusive.
#[tokio::test]
async fn a_signed_delivery_verifies_and_its_key_is_fetched_once() {
    let (signing, key_response) = keypair("kid-1");
    let (address, server) = serve(vec![(200, key_response)]);
    let verifier = WebhookVerifier::new(client(address));

    let first_body = br#"{"webhook_type":"ITEM","webhook_code":"ERROR","item_id":"a"}"#;
    let first = verifier
        .verify(
            Some(&token(&signing, "kid-1", now(), first_body)),
            first_body,
        )
        .await
        .expect("a delivery signed by plaid's key should verify");

    let second_body = br#"{"webhook_type":"ITEM","webhook_code":"ERROR","item_id":"b"}"#;
    let second = verifier
        .verify(
            Some(&token(&signing, "kid-1", now(), second_body)),
            second_body,
        )
        .await
        .expect("the key should have been cached rather than fetched again");

    // Two deliveries Plaid signed separately are two fingerprints, which is what
    // makes the fingerprint usable as a dedupe key.
    assert_ne!(first.fingerprint, second.fingerprint);

    server.join().expect("server thread");
}

/// A redelivery of the same signed token is the same fingerprint.
#[tokio::test]
async fn the_same_token_always_fingerprints_the_same() {
    let (signing, key_response) = keypair("kid-1");
    let (address, server) = serve(vec![(200, key_response)]);
    let verifier = WebhookVerifier::new(client(address));

    let body = br#"{"webhook_type":"ITEM"}"#;
    let token = token(&signing, "kid-1", now(), body);

    let first = verifier.verify(Some(&token), body).await.expect("first");
    let second = verifier
        .verify(Some(&token), body)
        .await
        .expect("redelivery");

    assert_eq!(first.fingerprint, second.fingerprint);

    server.join().expect("server thread");
}

/// A token signed by anything other than the key Plaid names is not Plaid's.
#[tokio::test]
async fn a_delivery_signed_with_the_wrong_key_is_refused() {
    let (_, key_response) = keypair("kid-1");
    let (address, server) = serve(vec![(200, key_response)]);
    let verifier = WebhookVerifier::new(client(address));

    let impostor = SigningKey::from_bytes(&[9u8; 32].into()).expect("a valid scalar");
    let body = br#"{"webhook_type":"ITEM"}"#;

    let error = verifier
        .verify(Some(&token(&impostor, "kid-1", now(), body)), body)
        .await
        .expect_err("a signature from another key must not verify");

    assert!(
        matches!(error, VerificationError::BadSignature),
        "{}",
        error
    );
    assert!(!error.is_inconclusive());

    server.join().expect("server thread");
}

/// The signature covers one body. Swapping the body after the fact is the whole
/// attack the digest claim exists to stop.
#[tokio::test]
async fn a_body_the_signature_does_not_cover_is_refused() {
    let (signing, key_response) = keypair("kid-1");
    let (address, server) = serve(vec![(200, key_response)]);
    let verifier = WebhookVerifier::new(client(address));

    let signed_body = br#"{"webhook_type":"ITEM","item_id":"real"}"#;
    let token = token(&signing, "kid-1", now(), signed_body);

    let error = verifier
        .verify(
            Some(&token),
            br#"{"webhook_type":"ITEM","item_id":"swapped"}"#,
        )
        .await
        .expect_err("a body the token does not cover must be refused");

    assert!(
        matches!(error, VerificationError::BodyMismatch),
        "{}",
        error
    );

    server.join().expect("server thread");
}

/// A signature stays valid for ever; the window is what stops a captured
/// delivery being replayed indefinitely.
#[tokio::test]
async fn a_delivery_outside_its_window_is_refused() {
    let (signing, key_response) = keypair("kid-1");
    let (address, server) = serve(vec![(200, key_response.clone()), (200, key_response)]);
    let verifier = WebhookVerifier::new(client(address));
    let body = br#"{"webhook_type":"ITEM"}"#;

    let old = verifier
        .verify(Some(&token(&signing, "kid-1", now() - 600, body)), body)
        .await
        .expect_err("a ten-minute-old delivery must be refused");
    assert!(matches!(old, VerificationError::Stale(_)), "{}", old);

    // A fresh verifier, so this one fetches the key rather than reusing it, and
    // the listener's second response is consumed either way.
    let verifier = WebhookVerifier::new(client(address));
    let future = verifier
        .verify(Some(&token(&signing, "kid-1", now() + 600, body)), body)
        .await
        .expect_err("a delivery from the future must be refused");
    assert!(matches!(future, VerificationError::Stale(_)), "{}", future);

    server.join().expect("server thread");
}

/// Plaid saying it does not sign with that key is a fact about the delivery, so
/// the delivery is refused rather than retried.
#[tokio::test]
async fn a_key_plaid_disowns_is_refused() {
    let rejection = json!({
        "error_type": "INVALID_INPUT",
        "error_code": "INVALID_FIELD",
        "error_message": "key_id must be a valid webhook verification key id",
        "display_message": serde_json::Value::Null,
        "request_id": "test-request-id",
        "documentation_url": serde_json::Value::Null,
        "suggested_action": serde_json::Value::Null,
    })
    .to_string();
    let (signing, _) = keypair("kid-1");
    let (address, server) = serve(vec![(400, rejection)]);
    let verifier = WebhookVerifier::new(client(address));

    let body = br#"{"webhook_type":"ITEM"}"#;
    let error = verifier
        .verify(Some(&token(&signing, "kid-1", now(), body)), body)
        .await
        .expect_err("a key plaid disowns must be refused");

    assert!(matches!(error, VerificationError::UnknownKey), "{}", error);
    assert!(
        !error.is_inconclusive(),
        "plaid answered, so the delivery was decided"
    );

    server.join().expect("server thread");
}

/// Not reaching Plaid decides nothing, so the delivery has to be asked for
/// again rather than dropped.
#[tokio::test]
async fn an_unreachable_plaid_is_inconclusive() {
    let (signing, _) = keypair("kid-1");
    let verifier = WebhookVerifier::new(client(nowhere()));

    let body = br#"{"webhook_type":"ITEM"}"#;
    let error = verifier
        .verify(Some(&token(&signing, "kid-1", now(), body)), body)
        .await
        .expect_err("an unreachable plaid cannot verify anything");

    assert!(
        matches!(error, VerificationError::KeyUnavailable(_)),
        "{}",
        error,
    );
    assert!(
        error.is_inconclusive(),
        "nothing was established, so the delivery must be retried"
    );
}

/// Everything a delivery can choose is checked before a call is made with it.
///
/// The client points at a closed port, so any of these reaching the fetch would
/// come back as [`VerificationError::KeyUnavailable`] instead.
#[tokio::test]
async fn what_the_sender_chooses_is_bounded_before_plaid_is_asked() {
    let verifier = WebhookVerifier::new(client(nowhere()));
    let body = br#"{}"#;

    let segment = |value: serde_json::Value| URL_SAFE_NO_PAD.encode(value.to_string());
    let claims =
        segment(json!({ "iat": now(), "request_body_sha256": hex::encode(Sha256::digest(body)) }));
    let unsigned = |header: serde_json::Value| format!("{}.{}.{}", segment(header), claims, "AAAA");

    // No header at all.
    assert!(matches!(
        verifier.verify(None, body).await.unwrap_err(),
        VerificationError::Missing
    ));

    // `alg: none` is the classic JWT hole: it must be refused, not honoured.
    let error = verifier
        .verify(
            Some(&unsigned(json!({ "alg": "none", "kid": "kid-1" }))),
            body,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(error, VerificationError::UnsupportedAlgorithm(ref alg) if alg == "none"),
        "{}",
        error,
    );

    // So is an asymmetric key presented as an HMAC secret.
    let error = verifier
        .verify(
            Some(&unsigned(json!({ "alg": "HS256", "kid": "kid-1" }))),
            body,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(error, VerificationError::UnsupportedAlgorithm(_)),
        "{}",
        error,
    );

    // The `kid` is what would be sent upstream, so the sender does not get to
    // choose an arbitrary string for it.
    for kid in [
        json!(""),
        json!("../../../etc/passwd"),
        json!("kid with spaces"),
        json!("k".repeat(65)),
    ] {
        let error = verifier
            .verify(Some(&unsigned(json!({ "alg": "ES256", "kid": kid }))), body)
            .await
            .unwrap_err();
        assert!(
            matches!(error, VerificationError::Malformed(_)),
            "`{}` should not have reached plaid: {}",
            kid,
            error,
        );
    }

    // And neither the shape of the token nor its size.
    for header in ["", "not-a-jwt", "one.two", "a.b.c.d", &"x".repeat(5000)] {
        let error = verifier.verify(Some(header), body).await.unwrap_err();
        assert!(
            matches!(error, VerificationError::Malformed(_)),
            "a token of {} bytes should not have reached plaid: {}",
            header.len(),
            error,
        );
    }
}

/// A key Plaid has retired verifies nothing, even for a token it really signed.
#[tokio::test]
async fn a_retired_key_is_refused() {
    let (signing, key_response) = keypair("kid-1");
    let mut key: serde_json::Value = serde_json::from_str(&key_response).expect("valid json");
    key["key"]["expired_at"] = json!(1_700_000_000);

    let (address, server) = serve(vec![(200, key.to_string())]);
    let verifier = WebhookVerifier::new(client(address));

    let body = br#"{"webhook_type":"ITEM"}"#;
    let error = verifier
        .verify(Some(&token(&signing, "kid-1", now(), body)), body)
        .await
        .expect_err("a retired key must verify nothing");

    assert!(matches!(error, VerificationError::KeyExpired), "{}", error);

    server.join().expect("server thread");
}
