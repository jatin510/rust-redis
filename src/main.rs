use std::str::from_utf8;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
};
// Uncomment this block to pass the first stage

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
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
        let output = commands
            .get(1)
            .map(|s| s.to_string()) // Convert &String to String if there is Some
            .unwrap_or_else(|| "".to_string());

        let command_upper = command.to_uppercase();

        match command_upper.as_str() {
            "PING" => {
                println!("PONG");
                let response = "+PONG\r\n";
                stream.write_all(response.as_bytes()).unwrap();
            }
            "ECHO" => {
                println!("output {}", output);
                let response = format!("${}\r\n{}\r\n", output.len(), output);
                stream.write_all(response.as_bytes()).unwrap();
            }
            _ => {
                println!("Something else is found ");
            }
        }
    }
}
