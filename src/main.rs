use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::env;
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
    expiry_time: Option<DateTime<Utc>>,
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

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let mut port = "6379";
    let binding = env::args().collect::<Vec<_>>();
    if let Some(port_str) = binding.get(2) {
        port = port_str
    } else {
    }

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(addr).unwrap();
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
                    let expire_time_in_ms = commands.get(4).unwrap();

                    store.set(key.clone(), value.clone(), Some(expire_time_in_ms.clone()))
                } else {
                    store.set(key.clone(), value.clone(), None);
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

            "INFO" => {
                let response = "$11\r\nrole:master\r\n".to_string();
                stream.write(response.as_bytes()).unwrap();
            }

            _ => {
                println!("Something else is found ");
            }
        }
    }
}
