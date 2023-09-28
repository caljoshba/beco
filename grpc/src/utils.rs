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
    user_id: Option<String>,
    end: Instant,
}

impl ProposeEvent {
    pub fn new(status: DataRequestType, user_id: Option<String>, duration: Duration) -> Self {
        Self {
            status: status,
            user_id,
            end: Instant::now() + duration,
        }
    }

    pub fn update(&mut self, status: DataRequestType, user_id: Option<String>) {
        self.status = status;
        if self.user_id.is_none() {
            self.user_id = user_id;
        }
    }
}

impl Future for ProposeEvent {
    type Output = (DataRequestType, Option<String>);
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.status == DataRequestType::VALIDATED
            || self.status == DataRequestType::FAILED
            || self.status == DataRequestType::RESPONSE
            || self.status == DataRequestType::LOAD
            || Instant::now() >= self.end
        {
            Poll::Ready((self.status.clone(), self.user_id.clone()))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl Future for &ProposeEvent {
    type Output = (DataRequestType, Option<String>);
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self.status == DataRequestType::VALIDATED
            || self.status == DataRequestType::FAILED
            || self.status == DataRequestType::RESPONSE
            || self.status == DataRequestType::LOAD
            || Instant::now() >= self.end
        {
            Poll::Ready((self.status.clone(), self.user_id.clone()))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
