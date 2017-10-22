use pad::PadStr;
use bincode::{serialize, deserialize, Infinite};

use std::net::UdpSocket;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Packet {
    data: Vec<u8>
}

pub fn start_node() {
    let mut socket = UdpSocket::bind("10.0.0.1:5001").unwrap();

    let mut buffer = [0; 32];

    loop {
        let (amt, src) = socket.recv_from(&mut buffer).unwrap();

        let buffer: Vec<u8> = buffer.to_vec();
        println!("{:?}", buffer);
        let packet: Packet = deserialize(&buffer[..]).unwrap();

        println!("{:?}", packet);

        // handle_request(buffer.to_vec());

        let buffer = [0; 32];
    }
}

fn handle_request(data: Vec<u8>) {
    println!("{:?}", data);
}
