#![feature(drain_filter)]

use futures::*;
use std::cmp::{Eq, Ord, Ordering, PartialOrd};
use std::collections::BinaryHeap;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[feature(async_closure)]
// #[derive(Copy, Clone)]
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
    pub callbacks: Vec<Entry>,
}

pub struct Scheduler {
    data: SharedData,
}
impl Scheduler {
    pub async fn tick(&mut self) {
        let now = Instant::now();

        let due_things: Vec<Entry> = self
            .data
            .callbacks
            .drain_filter(|x| x.instant <= now)
            .collect();

        for entry in due_things {
            (entry.callback)().await
        }
    }

    pub fn new() -> Scheduler {
        let shared_data = SharedData {
            cond_var: Condvar::new(),
            callbacks: Vec::new(),
        };
        {}

        Scheduler { data: shared_data }
    }

    pub fn after_instant(
        &mut self,
        instant: Instant,
        func: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    ) {
        self.data.callbacks.push(Entry {
            instant,
            callback: func,
        });
        self.data.cond_var.notify_all();
    }

    pub fn after_duration(
        &mut self,
        duration: Duration,
        func: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    ) {
        self.after_instant(Instant::now() + duration, func)
    }
}
