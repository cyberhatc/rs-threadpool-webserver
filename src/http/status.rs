/// HTTP status codes supported by this server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    NotFound = 404,
    InternalServerError = 500,
}

impl StatusCode {
    pub fn as_u16(self) -> u16 {
        self as u16
    }

    pub fn reason_phrase(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::BadRequest => "Bad Request",
            Self::NotFound => "Not Found",
            Self::InternalServerError => "Internal Server Error",
        }
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.as_u16(), self.reason_phrase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_status_line() {
        assert_eq!(StatusCode::Ok.to_string(), "200 OK");
        assert_eq!(StatusCode::NotFound.to_string(), "404 Not Found");
        assert_eq!(StatusCode::BadRequest.to_string(), "400 Bad Request");
        assert_eq!(
            StatusCode::InternalServerError.to_string(),
            "500 Internal Server Error"
        );
    }

    #[test]
    fn numeric_values() {
        assert_eq!(StatusCode::Ok.as_u16(), 200);
        assert_eq!(StatusCode::NotFound.as_u16(), 404);
    }
}
