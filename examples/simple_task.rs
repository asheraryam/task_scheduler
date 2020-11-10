use futures::*;
use std::cmp::{Eq, Ord, Ordering, PartialOrd};
use std::collections::BinaryHeap;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio;

extern crate task_scheduler;

// use futures::future;
use task_scheduler::Scheduler;

#[tokio::main]
async fn main() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let atomic = AtomicBool::new(false);
    let scheduler = Scheduler::new();

    scheduler.after_instant(
        Instant::now() + Duration::from_millis(10000),
        Box::new(|| {
            Box::pin(async {
                println!("This ran after 10 seconds!");
                // FIXME This does not live long enough
                // atomic.store(true, Ordering::Relaxed);
            })
        }),
    );

    scheduler.tick().await;

    // atomic.store(true, Ordering::Relaxed);

    // thread::sleep(Duration::from_millis(100));
    // assert_eq!(atomic.load(Ordering::Relaxed), true);
}
