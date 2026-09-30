//! Integration tests: spin up a real server on an ephemeral port and hit it
//! with raw TCP clients.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use rs_threadpool_webserver::Server;

fn start_server() -> (String, std::thread::JoinHandle<()>) {
    let listener_probe = TcpStream::connect("127.0.0.1:1").ok(); // ensure loopback up
    drop(listener_probe);

    // Bind to port 0 via a helper: Server::new needs an addr string.
    // We bind a throwaway listener to discover a free port, then drop it
    // and hand the same address to Server (race window is tiny on localhost).
    let probe = std::net::TcpListener::bind("127.0.0.1:0").expect("bind probe");
    let addr = probe.local_addr().expect("local addr");
    drop(probe);

    let server = Server::new(&addr.to_string(), 4, "public").expect("start server");
    let bound = server.local_addr().expect("server local addr").to_string();

    let handle = std::thread::spawn(move || {
        let _ = server.run();
    });

    // Wait until the port accepts connections.
    for _ in 0..50 {
        if TcpStream::connect(&bound).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    (bound, handle)
}

fn http_get(addr: &str, path: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).expect("write");
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).expect("read");
    let raw = String::from_utf8_lossy(&buf).to_string();
    let status = raw
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    (status, raw)
}

#[test]
fn serves_index_with_200() {
    let (addr, _handle) = start_server();
    let (status, body) = http_get(&addr, "/");
    assert_eq!(status, 200, "body: {body}");
    assert!(body.contains("rs-threadpool-webserver"), "body: {body}");
}

#[test]
fn serves_about_with_200() {
    let (addr, _handle) = start_server();
    let (status, body) = http_get(&addr, "/about");
    assert_eq!(status, 200, "body: {body}");
    assert!(body.contains("About this project"), "body: {body}");
}

#[test]
fn serves_css_content_type() {
    let (addr, _handle) = start_server();
    let (status, body) = http_get(&addr, "/style.css");
    assert_eq!(status, 200, "body: {body}");
    assert!(
        body.contains("Content-Type: text/css"),
        "missing css content-type: {body}"
    );
}

#[test]
fn missing_page_returns_404() {
    let (addr, _handle) = start_server();
    let (status, body) = http_get(&addr, "/does-not-exist");
    assert_eq!(status, 404, "body: {body}");
    assert!(body.contains("404"), "body: {body}");
}

#[test]
fn path_traversal_is_blocked() {
    let (addr, _handle) = start_server();
    let (status, _body) = http_get(&addr, "/../Cargo.toml");
    // Either rejected during routing or served as 404 — never 200 with source.
    assert_ne!(status, 200);
}

#[test]
fn concurrent_clients_are_handled() {
    let (addr, _handle) = start_server();
    let mut handles = Vec::new();
    for i in 0..16 {
        let a = addr.clone();
        handles.push(std::thread::spawn(move || {
            let path = if i % 2 == 0 { "/" } else { "/about" };
            let (status, _) = http_get(&a, path);
            assert_eq!(status, 200);
        }));
    }
    for h in handles {
        h.join().expect("client thread");
    }
}
