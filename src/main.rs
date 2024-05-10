use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::thread::yield_now;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
};
// Uncomment this block to pass the first stage

#[derive(Clone, Debug)]
pub struct RedisData {
    data: String,
    expiry_time: DateTime<Utc>,
    should_expire: bool,
}

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
            if redis_data.should_expire && Utc::now() > redis_data.expiry_time {
                data.remove(key);
                None
            } else {
                Some(redis_data.data.clone())
            }
        } else {
            None
        }
    }

    pub fn set(&self, key: String, value: String) {
        let mut data = self.data.lock().unwrap();

        let redis_data = RedisData {
            data: value,
            expiry_time: Utc::now(),
            should_expire: false,
        };

        data.insert(key, redis_data);
    }

    pub fn set_with_expire(&self, key: String, value: String, expire_after_ms: String) {
        let mut data = self.data.lock().unwrap();

        let redis_data = RedisData {
            data: value,
            expiry_time: Utc::now() + Duration::milliseconds(expire_after_ms.parse().unwrap()),
            should_expire: true,
        };

        data.insert(key.clone(), redis_data.clone());
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let mut store = Arc::new(Storage::new());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store_clone = store.clone();
                thread::spawn(move || {
                    handle_client(stream, store_clone);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, store: Arc<Storage>) {
    // let store_clone = store.clone();
    let mut buffer = [0; 1024];

    loop {
        let read_count = stream.read(&mut buffer).unwrap();
        let mut text = from_utf8(&buffer).unwrap();

        if read_count == 0 {
            break;
        }

        let mut lines = text.lines();
        let mut commands: Vec<String> = Vec::new();

        while let Some(line) = lines.next() {
            if line.starts_with("*") {
                // TODO
            } else if line.starts_with("$") {
                if let Some(data) = lines.next() {
                    commands.push(data.to_string());
                }
            }
        }

        let command = commands.get(0).unwrap();

        let command_upper = command.to_uppercase();

        match command_upper.as_str() {
            "PING" => {
                let output = "+PONG\r\n";
                stream.write_all(output.as_bytes()).unwrap();
            }
            "ECHO" => {
                let response = commands.get(1).unwrap();
                let output = format!("${}\r\n{}\r\n", response.len(), response);
                stream.write_all(output.as_bytes()).unwrap();
            }
            "SET" => {
                let key = commands.get(1).unwrap();
                let mut value = commands
                    .get(2)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "".to_string());

                if let Some(expire_command) = commands.get(3) {
                    let expire_time = commands.get(4).unwrap();

                    store.set_with_expire(key.clone(), value.clone(), expire_time.clone());
                } else {
                    store.set(key.clone(), value.clone());
                }

                let response = "+OK\r\n";
                stream.write_all(response.as_bytes()).unwrap();
            }
            "GET" => {
                let key = commands.get(1).unwrap();
                let value = store.get(key).unwrap_or_default();

                let mut response = format!("${}\r\n{}\r\n", value.len(), value);

                if value.len() == 0 {
                    response = "$-1\r\n".to_string();
                }
                stream.write_all(response.as_bytes()).unwrap();
            }

            _ => {
                println!("Something else is found ");
            }
        }
    }
}
