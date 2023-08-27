use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    task::Poll,
    time::{Duration, Instant},
};

use futures::Future;

use crate::enums::data_value::DataRequestType;

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Debug)]
pub struct ProposeEvent {
    status: DataRequestType,
    end: Instant,
}

impl ProposeEvent {
    pub fn new(status: DataRequestType, duration: Duration) -> Self {
        Self {
            status: status,
            end: Instant::now() + duration,
        }
    }

    pub fn update(&mut self, status: DataRequestType) {
        println!("updating value");
        self.status = status;
    }
}

impl Future for ProposeEvent {
    type Output = DataRequestType;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.status == DataRequestType::VALIDATED
            || self.status == DataRequestType::FAILED
            || Instant::now() >= self.end
        {
            Poll::Ready(self.status.clone())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl Future for &ProposeEvent {
    type Output = DataRequestType;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.status == DataRequestType::VALIDATED
            || self.status == DataRequestType::FAILED
            || Instant::now() >= self.end
        {
            Poll::Ready(self.status.clone())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
