use std::{
    io::{self, stdin, Read, Write},
    net::{Shutdown, TcpStream},
    sync::mpsc::{self, TryRecvError},
    thread,
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

        // Retrieve message history
        print!(
            "{}",
            read_history(&mut stream).expect("Failed to read history")
        );

        // Avoid blocking when checking for new messages
        stream
            .set_nonblocking(true)
            .expect("Failed to set stream to nonblocking");

        // Create a channel and a thread for reading stdin
        let (input_sender, input_receiver) = mpsc::channel::<String>();

        thread::spawn(move || loop {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).expect("Failed to read");
            input_sender.send(buf).expect("Failed to send");
        });

        const MSG_PREFIX: &str = "消息☆★☆";
        let mut info_requested = 0;

        loop {
            match input_receiver.try_recv() {
                Ok(msg) => {
                    let msg = msg.trim();
                    if msg == "#quit" {
                        break;
                    } else if msg == "#info" {
                        info_requested = 3;
                    }
                    if !msg.is_empty() {
                        // TODO: Correct machine ID
                        write_all_gbk(
                            &mut stream,
                            &format!("新消息☆★☆{username}[ID:{id}]：☆★☆{msg}☆★☆{id}☆★☆1234567"),
                        );
                    }
                }
                Err(TryRecvError::Empty) => match read_gbk(&mut stream) {
                    Ok(data) => {
                        if info_requested > 0 {
                            info_requested -= 1;
                            print!("{}", data);
                        } else {
                            assert!(data.starts_with(MSG_PREFIX));
                            print!("{}", &data[MSG_PREFIX.len()..]);
                        }
                    }
                    Err(err) => assert!(err.kind() == io::ErrorKind::WouldBlock),
                },
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            }
        }

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

fn read_history(stream: &mut TcpStream) -> io::Result<String> {
    let mut data: Vec<u8> = vec![];
    // #info is recognised by the server and is never sent out as messages,
    // so it's used here as a indicator for end of message history
    // TODO: Define a proper structure for message history in message protocol
    loop {
        let mut buf = [0u8; 4096];
        let size: usize;
        match stream.read(&mut buf) {
            Ok(len) => size = len,
            Err(err) => return Err(err),
        }
        data.append(&mut buf[0..size].to_vec());
        if contains(&buf[0..size], "#info".as_bytes()) {
            break;
        }
    }

    let (cow, encoding_used, had_errors) = GBK.decode(&data[..]);
    assert_eq!(encoding_used, GBK);
    assert!(!had_errors);
    Ok(cow.to_string())
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    // TODO: better approach?
    if haystack.len() < needle.len() {
        false
    } else {
        for i in 0..haystack.len() - needle.len() + 1 {
            if &haystack[i..i + needle.len()] == needle {
                return true;
            }
        }
        false
    }
}
