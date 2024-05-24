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

#[derive(Debug, Clone)]
pub enum RedisCommand {
    Ping,
    Echo(String),
    Set(String, String, Option<String>),
    Get(String),
    Info,
}

struct RedisMessage {}

fn handle_client(mut stream: TcpStream, store: Arc<Storage>, is_master: bool) {
    // let store_clone = store.clone();
    let mut buffer = [0; 1024];

    loop {
        let read_count = stream.read(&mut buffer).unwrap();
        let mut text = from_utf8(&buffer).unwrap();
        let mut text = text.trim_matches('\0');
        if read_count == 0 {
            break;
        }

        let mut commands: Vec<String> = Vec::new();
        commands.push(text.to_string());

        let redis_command = parse_input_commands(&commands);

        match redis_command.get(0).unwrap() {
            RedisCommand::Ping => {
                let output = "+PONG\r\n";
                stream.write_all(output.as_bytes()).unwrap();
            }

            RedisCommand::Echo(response) => {
                let output = format!("${}\r\n{}\r\n", response.len(), response);
                stream.write_all(output.as_bytes()).unwrap();
            }

            RedisCommand::Set(key, value, expire_time_in_ms) => {
                let value = value.clone();

                if let Some(expire_command) = expire_time_in_ms {
                    store.set(key.clone(), value, Some(expire_command.clone()))
                } else {
                    store.set(key.clone(), value, None);
                }

                let response = "+OK\r\n";
                stream.write_all(response.as_bytes()).unwrap();
            }

            RedisCommand::Get(key) => {
                let value = store.get(key).unwrap_or_default();

                let mut response = format!("${}\r\n{}\r\n", value.len(), value);

                if value.len() == 0 {
                    response = "$-1\r\n".to_string();
                }
                stream.write_all(response.as_bytes()).unwrap();
            }

            RedisCommand::Info => {
                println!("Info command is found");
                let mut response = "$87\r\nrole:master:".to_string();

                if !is_master {
                    response = "$86\r\nrole:slave:".to_string();
                }

                response.push_str("master_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb:");
                response.push_str("master_repl_offset:0\r\n");

                stream.write_all(response.as_bytes()).unwrap();
            }
            _ => {
                println!("Something else is found ");
            }
        }
    }
}

fn parse_input_commands(commands: &[String]) -> Vec<RedisCommand> {
    let mut parsed_commands = Vec::new();
    let mut iter = commands.iter();

    while let Some(command) = iter.next() {
        // let mut command_iter = command.lines();
        let mut command_iter_vector: Vec<_> = command.lines().collect();
        let command_upper = command_iter_vector.get(2).unwrap().to_uppercase();

        match command_upper.as_str() {
            "PING" => {
                parsed_commands.push(RedisCommand::Ping);
            }
            "ECHO" => {
                let response = command_iter_vector.get(4).unwrap();
                parsed_commands.push(RedisCommand::Echo(response.to_string()));
            }
            "SET" => {
                let key = command_iter_vector.get(4).unwrap();
                let value = command_iter_vector.get(6).unwrap();
                let expire_time_in_ms = command_iter_vector.get(10).map(|s| s.to_string());

                parsed_commands.push(RedisCommand::Set(
                    key.to_string(),
                    value.to_string(),
                    expire_time_in_ms,
                ));
            }
            "GET" => {
                let key = command_iter_vector.get(4).unwrap();
                parsed_commands.push(RedisCommand::Get(key.to_string()));
            }

            "INFO" => parsed_commands.push(RedisCommand::Info),
            _ => {
                println!("Unknown command is found {:?}", command_upper);
            }
        }
    }
    parsed_commands
}
