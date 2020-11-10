use futures::*;
use std::cmp::{Eq, Ord, Ordering, PartialOrd};
use std::collections::BinaryHeap;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[feature(async_closure)]

struct Entry {
    pub instant: Instant,
    pub callback: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Entry {}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.instant == other.instant
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.instant.cmp(&other.instant) {
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => Ordering::Equal,
        }
    }
}

struct SharedData {
    pub cond_var: Condvar,
    pub callbacks: Mutex<BinaryHeap<Entry>>,
}

pub struct Scheduler {
    data: Arc<SharedData>,
}

impl Scheduler {
    pub async fn tick(&self) {
        let shared_data = self.data.clone();
				let mut callbacks = shared_data.callbacks.lock().unwrap();
				loop {
					let entry = callbacks.pop();
					match entry {
						Some(mut entry) => {
							let now = Instant::now();
							if entry.instant > now {
								let wait_duration = entry.instant - now;
								callbacks.push(entry);
								callbacks = shared_data.cond_var
										.wait_timeout(callbacks, wait_duration).unwrap().0;
							} else {
								(entry.callback)().await
							}
						}
						None => {
							callbacks = shared_data.cond_var.wait(callbacks).unwrap();
						}
					}
				}
    }

    pub fn new() -> Scheduler {
        let shared_data = Arc::new(SharedData {
            cond_var: Condvar::new(),
            callbacks: Mutex::new(BinaryHeap::new()),
        });
        {}

        Scheduler { data: shared_data }
    }

    pub fn after_instant(
        &self,
        instant: Instant,
        func: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    ) {
        self.data.callbacks.lock().unwrap().push(Entry {
            instant,
            callback: func,
        });
        self.data.cond_var.notify_all();
    }

    pub fn after_duration(
        &self,
        duration: Duration,
        func: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    ) {
        self.after_instant(Instant::now() + duration, func)
    }
}
