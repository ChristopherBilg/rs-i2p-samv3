//
// This example demonstrates how repliable datagrams can be used with ri2p
//
// Both client and server create I2pRepliableSocket objects and the client
// thread is also given the destination of the server.
//
// Client then sends to the server's destination a datagram and the server
// can receive both the datagram, its size but also the destination of the
// sennder, in this case the client by calling recv_from().
//
// Server then sends a datagram to the destination it got from recv_from()
// and waits for another datagram from the client
//
use std::env;
use std::thread;
use std::time;

extern crate ri2p;

fn client(dest: String) {
    std::thread::sleep(time::Duration::from_millis(2000));

    // create raw i2p datagram socket and bind it to port 8888
    let mut socket = ri2p::proto::datagram::I2pRepliableSocket::new(8888).unwrap();
    let msg = "Hello, server".to_string();
    let mut buf = vec![0; 13];

    loop {
        // send message to server and response, then wait 5 seconds
        socket.send_to(msg.as_bytes(), &dest).unwrap();
        socket.recv_from(&mut buf).unwrap();

        println!("server sent: '{:#?}'", std::str::from_utf8(&buf).unwrap());

        std::thread::sleep(time::Duration::from_millis(5000));
    }
}

fn main() {

    // when I2pRawSocket is created, the router is notified that we are listening
    // to port 7777 for incoming datagrams
    //
    // currently, ri2p does not support the legacy v1/v2 way of accepting
    // incoming datagrams where they are routed through the control socket
    //
    // that is why port must be provided
    let mut socket = ri2p::proto::datagram::I2pRepliableSocket::new(7777).unwrap();
    let local_dest = socket.get_local_dest().to_string();
    let mut buf    = vec![0; 13];
    let msg        = "Hello, client".to_string();

    // spawn a thread for the client
    thread::spawn(move|| { client(local_dest) });

    loop {
        // read response (# of bytes read and address of the remote peer)
        // from the socket and respond
        let (_, addr) = socket.recv_from(&mut buf).unwrap();
        socket.send_to(msg.as_bytes(), &addr);

        println!("client sent: '{:#?}'", std::str::from_utf8(&buf).unwrap());
    }
}
