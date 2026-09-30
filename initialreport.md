Project Proposal \& Technical Specification Report: Multi-Threaded TCP Web Server in Rust

Prepared for: Open Source Community / OpenCDE



Project Name: rs-threadpool-webserver



Language: Rust (Edition 2021+)



Domain: Low-Level Systems Programming, Networking, Concurrency



1\. Executive Summary

Modern web applications and frameworks abstract away the underlying transport layers, thread management, and synchronization primitives. While this accelerates high-level application development, it often obscures a deep understanding of core systems engineering.



This project entails building a Multi-Threaded TCP Web Server from Scratch in Rust. By avoiding heavyweight web frameworks (such as Actix-web or Axum), this project implements a custom thread pool and leverages standard networking libraries to handle raw HTTP requests safely and concurrently. It serves as an educational and production-grade demonstration of Rust's ownership model, fearless concurrency, and safe memory management in a systems-programming context.



2\. Project Objectives \& Scope

Zero Heavy Frameworks: Build the server utilizing only Rust's standard library (std::net, std::thread, std::sync) to handle socket binding, listening, and thread orchestration.



Custom Thread Pool Architecture: Implement a reusable worker-pool pattern to avoid the performance overhead of spawning a new operating system thread for every incoming connection.



Safe Concurrency: Utilize Rust’s concurrency primitives (Arc, Mutex, channels) to safely distribute jobs across worker threads without data races.



HTTP/1.1 Compliance (Subset): Parse incoming HTTP request lines, handle basic routing, manage file I/O for static assets (e.g., HTML pages), and return well-formatted HTTP responses (Status 200 OK, 404 Not Found).



Robust Error Handling: Gracefully handle dropped connections, malformed payloads, and resource exhaustion without panicking.



3\. Technical Architecture \& Design

3.1 High-Level Architecture

Listener Loop (TcpListener): Binds to a local port (e.g., 127.0.0.1:7878) and listens for incoming TCP stream connections in a continuous loop.



Job Dispatcher: Upon accepting a connection, packages the TcpStream into a closure/job and dispatches it to the thread pool queue.



Thread Pool \& Workers:



A fixed number of worker threads are spawned at startup.



Workers block on an internal thread-safe receiver queue waiting for incoming tasks.



When a task arrives, a worker acquires the lock, extracts the job, executes the HTTP request-response cycle, and returns to waiting.



3.2 Core Components \& Rust Concepts

Networking (std::net::TcpListener \& TcpStream): Establishes low-level socket communication, reading byte streams from clients and writing responses back.



Concurrency Control (std::sync):



Arc<Mutex<mpsc::Receiver<Job>>>: Safely shares the job receiver across multiple worker threads, ensuring mutual exclusion when pulling tasks from the queue.



std::thread::spawn: Initializes background worker loops.



Drop Trait Implementation: Ensures that when the thread pool shuts down, all worker threads join cleanly, preventing dangling threads or resource leaks.



4\. Implementation Plan \& Milestones

Phase 1: Basic Single-Threaded TCP Listener



Initialize a Rust binary crate.



Bind a TcpListener to a port and print incoming raw byte requests to the console.



Construct and send a basic HTTP response string back over the stream.



Phase 2: Static File Serving \& Basic Parsing



Implement line-by-line or buffer-based reading of the HTTP request.



Parse the request path (e.g., / vs /slow-page).



Read static HTML files from disk and wrap them in a valid HTTP/1.1 header response (handling 404 states gracefully).



Phase 3: Custom Thread Pool Implementation



Design the ThreadPool struct and Worker structs.



Implement Multi-Producer, Single-Consumer (mpsc) channels for safe task handoff.



Wrap shared receiver state inside Arc<Mutex<...>>.



Phase 4: Integration, Testing \& Benchmarking



Refactor the server to hand incoming TcpStream instances directly to the thread pool.



Write integration tests simulating concurrent client connections using std::thread or benchmarking tools (like Apache Bench or wrk).



5\. Expected Deliverables on GitHub

Clean, Idiomatic Rust Code: Fully documented source code structured cleanly into modules (lib.rs for server/pool logic, main.rs for entry-point configuration).



Comprehensive README.md: Detailed instructions on building, running, testing, and benchmarking the server locally.



Sample Assets: A set of HTML templates (index.html, 404.html) to demonstrate dynamic static-file routing.



CI/CD Pipeline: GitHub Actions configuration running cargo test, cargo clippy, and cargo fmt checks automatically on pull requests.



6\. Project Value \& Open-Source Impact

This project bridges the gap between high-level web concepts and low-level operating system interfaces. For the open-source community, it provides a transparent, readable reference implementation for how network engines and thread pools operate beneath popular web frameworks, serving as an ideal learning resource for engineers transitioning into systems programming and Rust.

