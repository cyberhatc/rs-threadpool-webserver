//! TCP listener loop, request routing, and static file serving.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

use crate::http::{HttpRequest, HttpResponse, StatusCode};
use crate::threadpool::ThreadPool;

/// Multi-threaded HTTP server backed by a custom thread pool.
pub struct Server {
    listener: TcpListener,
    pool: ThreadPool,
    root: PathBuf,
}

impl Server {
    /// Bind `addr`, create a pool of `pool_size` workers, and serve files from `root`.
    pub fn new(addr: &str, pool_size: usize, root: impl Into<PathBuf>) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let pool = ThreadPool::new(pool_size);
        Ok(Self {
            listener,
            pool,
            root: root.into(),
        })
    }

    /// Local address the server is bound to (useful when port 0 was requested).
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Accept connections forever, dispatching each to the thread pool.
    pub fn run(self) -> std::io::Result<()> {
        let addr = self.listener.local_addr()?;
        println!("rs-threadpool-webserver listening on http://{addr}");
        println!("Serving static files from: {}", self.root.display());
        println!("Ctrl+C to stop.");

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let root = self.root.clone();
                    if let Err(e) = self.pool.execute(move || {
                        if let Err(e) = handle_connection(stream, &root) {
                            eprintln!("connection error: {e}");
                        }
                    }) {
                        eprintln!("failed to enqueue connection: {e}");
                    }
                }
                Err(e) => eprintln!("accept failed: {e}"),
            }
        }

        Ok(())
    }

    /// Number of worker threads.
    pub fn pool_size(&self) -> usize {
        self.pool.size()
    }
}

fn handle_connection(mut stream: TcpStream, root: &Path) -> std::io::Result<()> {
    let mut buffer = [0u8; 8 * 1024];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(());
    }

    let raw = String::from_utf8_lossy(&buffer[..bytes_read]);
    let request = match HttpRequest::parse(&raw) {
        Some(req) => req,
        None => {
            let response = HttpResponse::new(StatusCode::BadRequest, "Bad Request");
            return stream.write_all(&response.to_bytes());
        }
    };

    println!("{} {} {}", request.method, request.path, request.version);

    let response = route(&request, root);
    stream.write_all(&response.to_bytes())
}

fn route(request: &HttpRequest, root: &Path) -> HttpResponse {
    if request.method != "GET" && request.method != "HEAD" {
        return HttpResponse::new(StatusCode::BadRequest, "Method Not Allowed");
    }

    let path = request.path.split('?').next().unwrap_or("/");

    match path {
        "/" => serve_file(root, "index.html"),
        "/404" => serve_file(root, "404.html"),
        p => {
            let clean = p.trim_start_matches('/');
            if clean.is_empty() || clean.contains("..") || clean.contains('\\') {
                return HttpResponse::new(StatusCode::NotFound, "Not Found");
            }
            // Prefer exact file (e.g. /style.css); otherwise try <name>.html.
            let as_file = root.join(clean);
            if as_file.is_file() {
                serve_file(root, clean)
            } else if !Path::new(clean).extension().is_some() {
                serve_file(root, &format!("{clean}.html"))
            } else {
                serve_file(root, clean)
            }
        }
    }
}

fn serve_file(root: &Path, relative: &str) -> HttpResponse {
    let path = root.join(relative);

    // Reject path traversal outside root.
    if !path.starts_with(root) {
        return HttpResponse::new(StatusCode::NotFound, "Not Found");
    }

    match std::fs::read(&path) {
        Ok(body) => {
            let content_type = content_type_for(&path);
            HttpResponse::new(StatusCode::Ok, body).with_header("Content-Type", content_type)
        }
        Err(_) => {
            let body = std::fs::read(root.join("404.html")).unwrap_or_else(|_| {
                b"<!DOCTYPE html><html><body><h1>404 Not Found</h1></body></html>".to_vec()
            });
            HttpResponse::new(StatusCode::NotFound, body)
        }
    }
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}
