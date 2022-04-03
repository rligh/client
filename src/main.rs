use std::net::{TcpStream, Shutdown};

fn main() {
    let stream =
        TcpStream::connect("101.35.253.48:19730").expect("Failed to connect to server");

    stream.shutdown(Shutdown::Both).expect("Failed to shutdown");
}
