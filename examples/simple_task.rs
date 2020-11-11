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
    let mut scheduler = Scheduler::new();

    scheduler.after_instant(
        Instant::now() + Duration::from_millis(5000),
        Box::new(|| {
            Box::pin(async {
                println!("This ran after 5 seconds!");
                // FIXME This does not live long enough
                // atomic.store(true, Ordering::Relaxed);
            })
        }),
    );

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

    loop {
        println!("Tick ");
        scheduler.tick().await;
        thread::sleep(Duration::from_millis(1000));
    }
}
