//! # rs-threadpool-webserver
//!
//! A multi-threaded TCP web server built from scratch in Rust using only the
//! standard library (`std::net`, `std::thread`, `std::sync`).
//!
//! ## Modules
//!
//! - [`http`] — HTTP request parsing, response building, and status codes
//! - [`server`] — TCP listener loop and connection handling
//! - [`threadpool`] — reusable worker-pool concurrency primitive

pub mod http;
pub mod server;
pub mod threadpool;

pub use http::{HttpRequest, HttpResponse, StatusCode};
pub use server::Server;
pub use threadpool::ThreadPool;
