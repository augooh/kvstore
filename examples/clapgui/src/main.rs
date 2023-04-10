use std::io::{self, Write}; // 引入标准I/O库
use clap::{Arg, App}; // 引入clap库

fn main() {
    let matches = App::new("redis interactive shell")
        .version("1.0")
        .author("Your Name <you@example.com>")
        .about("A Redis-like interactive shell")
        .get_matches();

    loop {
        print!("kvstore> ");
        io::stdout().flush().unwrap(); // 确保打印内容能够立即显示

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap(); // 读取用户的输入

        let command = input.trim(); // 去掉输入字符串两端的空白字符

        if command.to_lowercase() == "exit" {
            break; // 如果用户输入"exit"则退出循环
        }

        // 在这里执行用户的命令，例如将命令发送给Redis服务器进行处理
        println!("You entered: {}", command); // 输出用户的输入
    }
}