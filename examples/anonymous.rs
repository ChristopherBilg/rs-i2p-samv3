use std::env;
use std::thread;
use std::time;

extern crate ri2p;

fn client(dest: String) {
    std::thread::sleep(time::Duration::from_millis(2000));

    // Create raw i2p datagram socket and bind it to port 8888
    let mut socket = ri2p::proto::datagram::I2pRawSocket::new(8888).unwrap();
    let msg = "Hello, world!".to_string();

    loop {
        socket.send_to(msg.as_bytes(), &dest).unwrap();
        std::thread::sleep(time::Duration::from_millis(5000));
    }
}

fn main() {

    // When I2pRawSocket is created, the router is notified that we are listening
    // to port 7777 for incoming datagrams
    //
    // Currently, ri2p does not support the legacy v1/v2 way of accepting
    // incoming datagrams where they are routed through the control socket.
    // That is why port must be provided.
    let mut socket = ri2p::proto::datagram::I2pRawSocket::new(7777).unwrap();
    let local_dest = socket.get_local_dest().to_string();

    // spawn a thread for the client
    thread::spawn(move|| { client(local_dest) });

    println!("waiting for an incoming raw datagrams...");

    loop {
        let mut buf = vec![0; 13];

        match socket.recv(&mut buf) {
            Ok(_)  => println!("client sent: '{:#?}'", std::str::from_utf8(&buf).unwrap()),
            Err(e) => eprintln!("failure: {:#?}", e),
        }
    }
}
