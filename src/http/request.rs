use std::collections::HashMap;

/// A parsed HTTP/1.x request.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    /// Parse a raw HTTP request from a UTF-8 lossy string.
    ///
    /// Returns `None` if the request line is missing required parts.
    pub fn parse(raw: &str) -> Option<Self> {
        let mut lines = raw.split("\r\n");

        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let path = parts.next()?.to_string();
        let version = parts.next()?.to_string();
        if parts.next().is_some() {
            return None;
        }
        if method.is_empty()
            || !method.chars().all(|c| c.is_ascii_alphabetic())
            || !path.starts_with('/')
            || !version.starts_with("HTTP/")
        {
            return None;
        }

        let mut headers = HashMap::new();
        for line in lines.by_ref() {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        let body = lines.collect::<Vec<_>>().join("\n");

        Some(Self {
            method,
            path,
            version,
            headers,
            body,
        })
    }

    /// Look up a header value (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_get_request() {
        let raw = "GET /index.html HTTP/1.1\r\nHost: localhost:7878\r\nAccept: text/html\r\n\r\n";
        let req = HttpRequest::parse(raw).expect("parse should succeed");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/index.html");
        assert_eq!(req.version, "HTTP/1.1");
        assert_eq!(req.header("host"), Some("localhost:7878"));
        assert_eq!(req.header("Host"), Some("localhost:7878"));
        assert_eq!(req.body, "");
    }

    #[test]
    fn parses_post_with_body() {
        let raw = "POST /submit HTTP/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: 11\r\n\r\nname=dev\r\n";
        let req = HttpRequest::parse(raw).expect("parse should succeed");
        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/submit");
        assert_eq!(
            req.header("content-type"),
            Some("application/x-www-form-urlencoded")
        );
        assert!(req.body.contains("name=dev"));
    }

    #[test]
    fn returns_none_for_garbage() {
        assert!(HttpRequest::parse("not a request").is_none());
        assert!(HttpRequest::parse("").is_none());
    }
}
