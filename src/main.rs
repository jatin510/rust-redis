use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
// Uncomment this block to pass the first stage

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok( stream) => {
                 handle_client(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream:TcpStream){
    let mut buffer = [0;1024];

    loop {
        let read_count = stream.read(&mut buffer).unwrap();
        if read_count == 0 {
            break;
        }

        stream.write_all("+PONG\r\n".as_bytes());
    }
}

// let buf_reader = BufReader::new(&mut stream);

// for line_result in buf_reader.lines() {
//     let line = line_result.unwrap();
//
//     println!("line {}", line);
//     if line == "PING"{
//         let response = "+PONG\r\n";
//         println!("PING is received");
//         //
//         continue;
//     }
//     else{
//         // break;
//         continue;
//     }
// }
