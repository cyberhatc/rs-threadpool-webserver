use rs_threadpool_webserver::Server;

fn main() {
    let addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:7878".to_string());
    let pool_size = std::env::var("POOL_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(4);
    let root = std::env::var("PUBLIC_DIR").unwrap_or_else(|_| "public".to_string());

    let server = match Server::new(&addr, pool_size, &root) {
        Ok(server) => server,
        Err(e) => {
            eprintln!("failed to start server on {addr}: {e}");
            std::process::exit(1);
        }
    };

    println!("pool size: {} | public dir: {root}", server.pool_size());

    if let Err(e) = server.run() {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
}
