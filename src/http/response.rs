use std::collections::HashMap;

use super::status::StatusCode;

/// An HTTP response ready to be written to a TCP stream.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: StatusCode, body: impl Into<Vec<u8>>) -> Self {
        let mut headers = HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );
        Self {
            status,
            headers,
            body: body.into(),
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn html(body: &str) -> Self {
        Self::new(StatusCode::Ok, body.as_bytes().to_vec())
    }

    /// Serialize to a full HTTP/1.1 message (status line + headers + body).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut headers = self.headers.clone();
        headers.insert("Content-Length".to_string(), self.body.len().to_string());
        headers.insert("Connection".to_string(), "close".to_string());

        let mut response = format!("HTTP/1.1 {}\r\n", self.status);
        for (key, value) in &headers {
            response.push_str(&format!("{key}: {value}\r\n"));
        }
        response.push_str("\r\n");

        let mut bytes = response.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_html_response() {
        let resp = HttpResponse::html("<h1>Hello</h1>");
        let raw = String::from_utf8(resp.to_bytes()).unwrap();
        assert!(raw.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(raw.contains("Content-Type: text/html; charset=utf-8"));
        assert!(raw.contains("Content-Length: 14"));
        assert!(raw.contains("Connection: close"));
        assert!(raw.ends_with("\r\n\r\n<h1>Hello</h1>"));
    }

    #[test]
    fn serializes_404() {
        let resp = HttpResponse::new(StatusCode::NotFound, "nope");
        let raw = String::from_utf8(resp.to_bytes()).unwrap();
        assert!(raw.starts_with("HTTP/1.1 404 Not Found\r\n"));
        assert!(raw.contains("Content-Length: 4"));
    }

    #[test]
    fn overrides_content_type() {
        let resp = HttpResponse::new(StatusCode::Ok, b"body".to_vec())
            .with_header("Content-Type", "text/plain");
        let raw = String::from_utf8(resp.to_bytes()).unwrap();
        assert!(raw.contains("Content-Type: text/plain"));
        assert!(!raw.contains("text/html"));
    }
}
