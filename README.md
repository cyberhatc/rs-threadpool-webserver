# rs-threadpool-webserver

A multi-threaded TCP web server built from scratch in **Rust** using only the
standard library — no Actix, Axum, Hyper, or Tokio.

It implements a reusable **thread pool**, **HTTP/1.1 request parsing**, and
**static file serving** with graceful error handling.

## Features

- Custom fixed-size worker pool (`Arc<Mutex<mpsc::Receiver>>`)
- TCP listener + job dispatcher on `std::net::TcpListener`
- HTTP request line / header parsing
- Static file serving with `Content-Type` detection
- Path-traversal protection (`..`, `\`)
- 200 OK, 400 Bad Request, 404 Not Found responses
- Clean worker shutdown via `Drop` (join all threads)
- CI: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`

## Project layout

```
.
├── Cargo.toml
├── public/                 # Static assets
│   ├── index.html
│   ├── about.html
│   ├── 404.html
│   └── style.css
├── src/
│   ├── main.rs             # CLI entry point / env config
│   ├── lib.rs              # Library root
│   ├── server.rs           # Listener loop + routing
│   ├── threadpool/mod.rs   # ThreadPool + Worker
│   └── http/
│       ├── mod.rs
│       ├── request.rs
│       ├── response.rs
│       └── status.rs
├── tests/
│   └── server_test.rs      # Integration tests
└── .github/workflows/ci.yml
```

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run --release
```

Default bind address: `127.0.0.1:7878`  
Default pool size: `4`  
Default static root: `public/`

Override with environment variables:

```bash
# PowerShell
$env:SERVER_ADDR = "127.0.0.1:9000"
$env:POOL_SIZE = "8"
$env:PUBLIC_DIR = "public"
cargo run --release

# bash
SERVER_ADDR=127.0.0.1:9000 POOL_SIZE=8 PUBLIC_DIR=public cargo run --release
```

Then open <http://127.0.0.1:7878/> in a browser, or:

```bash
curl -i http://127.0.0.1:7878/
curl -i http://127.0.0.1:7878/about
curl -i http://127.0.0.1:7878/missing-page
curl -i -X POST http://127.0.0.1:7878/
```

## Test

```bash
cargo test
```

## Lint & format

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

## Benchmark (optional)

With the server running:

```bash
# Apache Bench (example)
ab -n 2000 -c 50 http://127.0.0.1:7878/

# wrk (example)
wrk -t4 -c100 -d10s http://127.0.0.1:7878/
```

## How it works

1. **`Server::run`** accepts TCP connections on a `TcpListener`.
2. Each accepted `TcpStream` is wrapped in a closure and sent to the **thread pool**.
3. Workers block on `mpsc::Receiver` (shared via `Arc<Mutex<_>>`), dequeue a job, and run the HTTP cycle.
4. The handler parses the request, routes the path, reads a file from `public/`, and writes a well-formed HTTP response.
5. Dropping the pool closes the channel and **joins** all workers.

## License

MIT
