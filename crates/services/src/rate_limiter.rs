use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct RateLimitBucket {
    pub window_start: Instant,
    pub request_count: u32,
}

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, RateLimitBucket>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns true if the request is allowed.
    pub fn allow(&self, api_key_id: &str, limit_per_minute: i32) -> bool {
        let mut buckets = self.buckets.lock().unwrap();

        let bucket = buckets
            .entry(api_key_id.to_string())
            .or_insert(RateLimitBucket {
                window_start: Instant::now(),
                request_count: 0,
            });

        if bucket.window_start.elapsed() >= Duration::from_secs(60) {
            bucket.window_start = Instant::now();
            bucket.request_count = 0;
        }

        if bucket.request_count >= limit_per_minute as u32 {
            return false;
        }

        bucket.request_count += 1;

        true
    }
}