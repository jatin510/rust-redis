mod storage;

use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use std::collections::HashMap;
use std::env;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
};

use storage::Storage;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 6379)]
    port: u16,
    #[arg(short, long)]
    replicaof: Option<Vec<String>>,
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    let cmd_args = Arc::new(Args::parse());

    let addr = format!("127.0.0.1:{}", cmd_args.port);
    let is_master = cmd_args.replicaof.is_none();

    let listener = TcpListener::bind(addr).unwrap();
    let mut store = Arc::new(Storage::new());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store_clone = store.clone();
                thread::spawn(move || {
                    handle_client(stream, store_clone, is_master);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, store: Arc<Storage>, is_master: bool) {
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
                let mut response = "$87\r\nrole:master:".to_string();

                if !is_master {
                    response = "$86\r\nrole:slave:".to_string();
                }

                response.push_str("master_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb:");
                response.push_str("master_repl_offset:0\r\n");

                stream.write(response.as_bytes()).unwrap();
            }

            _ => {
                println!("Something else is found ");
            }
        }
    }
}
