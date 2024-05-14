use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Storage {
    data: Arc<Mutex<HashMap<String, RedisData>>>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut data = self.data.lock().unwrap();
        if let Some(redis_data) = data.get(key) {
            if let Some(expiry_time) = redis_data.expiry_time {
                if expiry_time < Utc::now() {
                    data.remove(key);
                    return None;
                }
                Some(redis_data.data.clone())
            } else {
                Some(redis_data.data.clone())
            }
        } else {
            None
        }
    }

    pub fn set(&self, key: String, value: String, expire_after_ms: Option<String>) {
        let mut data = self.data.lock().unwrap();

        let expiry_time = expire_after_ms
            .and_then(|ms_str| ms_str.parse::<i64>().ok())
            .map(|ms| Utc::now() + Duration::milliseconds(ms));

        let redis_data = RedisData {
            data: value,
            expiry_time,
        };

        data.insert(key, redis_data);
    }
}

#[derive(Clone, Debug)]
pub struct RedisData {
    data: String,
    expiry_time: Option<DateTime<Utc>>,
}
