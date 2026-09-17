//! Checks that [`plaid::Environment::Custom`] actually routes requests to the
//! given base URL, by serving one request from a local listener.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

/// Serves a single request, returning the request line and body it received.
fn serve_once(listener: TcpListener, response_body: String) -> (String, String) {
    let (mut stream, _) = listener.accept().expect("accept");
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));

    let mut request_line = String::new();
    reader.read_line(&mut request_line).expect("request line");

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

    let mut body = vec![0; content_length];
    reader.read_exact(&mut body).expect("body");

    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        response_body.len(),
        response_body,
    )
    .expect("respond");
    stream.flush().expect("flush");

    (
        request_line.trim().to_string(),
        String::from_utf8(body).expect("utf-8 body"),
    )
}

#[tokio::test]
async fn requests_go_to_the_custom_base_url() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("local addr");

    let response_body = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/accounts_get.json"
    ))
    .expect("fixture");

    let server = std::thread::spawn(move || serve_once(listener, response_body));

    let client = plaid::Client::new(
        "test_client_id",
        "test_secret".to_string(),
        // a trailing slash must not produce `//accounts/get`
        plaid::Environment::Custom(format!("http://{}/", address)),
    );

    let response = client
        .accounts("test-access-token")
        .await
        .expect("the local server should have answered");

    let (request_line, request_body) = server.join().expect("server thread");

    assert_eq!(request_line, "POST /accounts/get HTTP/1.1");
    assert!(
        request_body.contains("test-access-token"),
        "credentials should still be sent: {}",
        request_body,
    );
    assert!(!response.accounts.is_empty());
}
