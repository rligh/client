use std::{
    io::{self, stdin, Read, Write},
    net::{Shutdown, TcpStream},
};

use encoding_rs::GBK;

fn main() {
    let mut stream =
        TcpStream::connect("101.35.253.48:19730").expect("Failed to connect to server");

    // Read ID
    println!("ID：");
    let mut id = String::new();
    stdin().read_line(&mut id).expect("Failed to read ID");
    let id = id.trim();

    // Read password
    println!("密码：");
    let mut password = String::new();
    stdin().read_line(&mut password).expect("Failed to read ID");
    let password = password.trim();

    // Log in
    write_all_gbk(&mut stream, &format!("登录☆★☆{id}☆★☆{password}"));
    let res = read_gbk(&mut stream).expect("Failed to read username");
    if res == "错误☆★☆-1" {
        println!("ID或密码错误！");
        return;
    } else {
        const LOGIN_PREFIX: &str = "登录☆★☆";
        assert!(res.starts_with(LOGIN_PREFIX));
        let username = &res[LOGIN_PREFIX.len()..];
        println!("{}，欢迎！", username);

        // Request message history
        write_all_gbk(&mut stream, "历史信息☆★☆获取");

        // Log out
        write_all_gbk(&mut stream, &format!("下线★☆★{id}"));
    }

    stream.shutdown(Shutdown::Both).expect("Failed to shutdown");
}

fn write_all_gbk(stream: &mut TcpStream, data: &str) {
    let (buf, encoding_used, had_errors) = GBK.encode(data);
    assert_eq!(encoding_used, GBK);
    assert!(!had_errors);

    stream.write_all(&buf).expect("Failed to write");
}

fn read_gbk(stream: &mut TcpStream) -> io::Result<String> {
    let mut buf = [0u8; 4096];

    match stream.read(&mut buf) {
        Ok(len) => {
            let (cow, encoding_used, had_errors) = GBK.decode(&buf[0..len]);
            assert_eq!(encoding_used, GBK);
            assert!(!had_errors);
            Ok(cow.to_string())
        }
        Err(err) => Err(err),
    }
}
