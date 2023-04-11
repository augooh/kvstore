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

                loop {
                    let bytes_read = stream.read(&mut buffer).unwrap();
                    if bytes_read == 0 {
                        break;
                    }

                    let command = String::from_utf8_lossy(&buffer[..bytes_read]).trim().to_owned();
                    let response = process_command(&mut db, command);

                    stream.write_all(response.as_bytes()).unwrap();
                    stream.flush().unwrap();
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

    match tokens[0] {
        "SET" => {
            let key = tokens[1];
            let value = tokens[2..].join(" ");
            db.set(key, &value);
            "OK".to_owned()
        }
        "GET" => {
            let key = tokens[1];
            match db.get::<String>(&key) {
                Some(value) => value.to_owned(),
                None => "nil".to_owned(),
            }
        }
        "DEL" => {
            let key = tokens[1];
            if db.exists(&key) {
                db.rem(&key);
                "OK".to_owned()
            } else {
                "nil".to_owned()
            }
        }
        _ => "Invalid command".to_owned(),
    }
}