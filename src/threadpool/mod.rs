//! Reusable fixed-size worker pool for concurrent job execution.
//!
//! Uses `std::sync::mpsc` for a multi-producer, single-consumer job queue,
//! wrapped in `Arc<Mutex<Receiver>>` so multiple workers can pull tasks safely.

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// A fixed-size pool of worker threads that execute queued closures.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a pool with `size` worker threads.
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero.
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "ThreadPool size must be greater than 0");

        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        Self {
            workers,
            sender: Some(sender),
        }
    }

    /// Queue a job for execution on a worker thread.
    ///
    /// Returns `Err` if the pool has been shut down (all workers exited).
    pub fn execute<F>(&self, f: F) -> Result<(), mpsc::SendError<Job>>
    where
        F: FnOnce() + Send + 'static,
    {
        match &self.sender {
            Some(sender) => sender.send(Box::new(f)),
            None => Err(mpsc::SendError(Box::new(f))),
        }
    }

    /// Number of worker threads in the pool.
    pub fn size(&self) -> usize {
        self.workers.len()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Close the channel so blocked workers wake up and exit.
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(handle) = worker.thread.take() {
                let _ = handle.join();
            }
        }
    }
}

struct Worker {
    #[allow(dead_code)]
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let job = {
                // Hold the lock only while calling recv(), then drop it so
                // other workers can dequeue the next job during execution.
                let lock_result = receiver.lock();
                match lock_result {
                    Ok(rx) => rx.recv(),
                    Err(poisoned) => poisoned.into_inner().recv(),
                }
            };

            match job {
                Ok(job) => job(),
                // Channel closed and queue drained — time to exit.
                Err(_) => break,
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;
    use std::time::Duration;

    #[test]
    fn executes_jobs() {
        let counter = StdArc::new(AtomicUsize::new(0));
        let pool = ThreadPool::new(4);

        for _ in 0..100 {
            let c = StdArc::clone(&counter);
            pool.execute(move || {
                c.fetch_add(1, Ordering::SeqCst);
            })
            .expect("job should enqueue");
        }

        drop(pool);
        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn size_reports_worker_count() {
        let pool = ThreadPool::new(8);
        assert_eq!(pool.size(), 8);
    }

    #[test]
    #[should_panic(expected = "ThreadPool size must be greater than 0")]
    fn panics_on_zero_size() {
        let _ = ThreadPool::new(0);
    }

    #[test]
    fn jobs_run_after_drop_of_previous_pool() {
        let first = ThreadPool::new(2);
        drop(first); // must join cleanly

        let pool2 = ThreadPool::new(2);
        let ok = StdArc::new(AtomicUsize::new(0));
        let c = StdArc::clone(&ok);
        pool2
            .execute(move || {
                c.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(ok.load(Ordering::SeqCst), 1);
    }
}
