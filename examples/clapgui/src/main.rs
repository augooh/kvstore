use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:4567").expect("Could not connect to server");
    let mut input = String::new();
    let mut reader = BufReader::new(stream.try_clone().expect("Could not clone stream"));

    loop {
        print!("kvstore> ");
        std::io::stdout().flush().unwrap();

        // 从用户获取输入
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();
        let mut input_string = input.to_owned();

        // 发送命令到服务端
        stream
            .write_all(input_string.as_bytes())
            .expect("Failed to write to server");
        stream.write_all(b"\n").expect("Failed to write newline");

        // 读取服务端的响应并显示
        let mut buffer: Vec<u8> = Vec::new();
        reader
            .read_until(b'\n', &mut buffer)
            .expect("Could not read from server");
        let response = str::from_utf8(&buffer).expect("Invalid UTF-8");
        println!("{}", response.trim());

        input_string.clear();
    }
}
