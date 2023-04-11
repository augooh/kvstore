use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use kvstore::{KeyValueDb, KeyValueDbDumpPolicy, SerializationMethod};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4567").unwrap();
    println!("Server listening on 127.0.0.1:4567");

    let mut db = KeyValueDb::new(
        "keyvaluedb.db",
        KeyValueDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );

    let mut buffer = [0; 1024]; // 将 buffer 的定义移动到更高的作用域中

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let mut data_received = false;

                loop {
                    if !data_received {
                        println!("[+] processing");
                        let bytes_read = stream.read(&mut buffer).unwrap();
                        if bytes_read == 0 {
                            break;
                        }

                        let command = String::from_utf8_lossy(&buffer[..bytes_read]).trim().to_owned();
                        println!("cmd: {:?}", command);
                        let response = process_command(&mut db, command);

                        stream.write_all(response.as_bytes()).unwrap();
                        println!("rsp: {:?}", response);
                        stream.flush().unwrap();

                        data_received = true;
                    } else {
                        println!("[+] waiting");
                        // Check if there is new data available
                        let mut buf = [0; 1];
                        match stream.peek(&mut buf) {
                            Ok(0) => break, // No new data, break the loop
                            Ok(_) => { 
                                if let Ok(s) = std::str::from_utf8(&buf) {
                                    println!("buf: {:?}", s);
                                }
                                data_received = false;
                                continue;
                            },
                            Err(_) => break, // Error reading data, break the loop
                        }
                    }
                }
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}



fn process_command(db: &mut KeyValueDb, command: String) -> String {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    let mut response = String::new();

    match tokens[0] {
        "SET" => {
            let key = tokens[1];
            let value = tokens[2..].join(" ");
            db.set(key, &value);
            response = "OK".to_owned()
        }
        "GET" => {
            let key = tokens[1];
            match db.get::<String>(&key) {
                Some(value) => response = value.to_owned(),
                None => response = "nil".to_owned(),
            }
        }
        "DEL" => {
            let key = tokens[1];
            if db.exists(&key) {
                db.rem(&key);
                response = "OK".to_owned()
            } else {
                response = "nil".to_owned()
            }
        }
        _ => response = "Invalid command".to_owned(),
    }

    response.push('\n');
    response
}