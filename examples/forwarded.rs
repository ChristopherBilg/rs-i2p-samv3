//
// This example demonstrates how an I2P virtual stream can be
// forwarded to a normal Rust socket server
//
use std::env;
use std::thread;
use std::time;
use std::net::{TcpStream, TcpListener};
use std::io::{BufReader, BufWriter, Write, BufRead, Read};
use std::time::Duration;

extern crate ri2p;

fn client(dest: String) {
    std::thread::sleep(time::Duration::from_millis(2000));

    let mut stream = ri2p::proto::stream::I2pStream::new().unwrap();
    stream.connect(&dest).unwrap();

    loop {
        stream.write("Hello, world!\n".as_bytes()).unwrap();
        std::thread::sleep(time::Duration::from_millis(5000));
    }
}

fn main() {

    // create a new virtual stream session and tell the router that all
    // incoming connection requests should be forwarded to local TCP server
    // listening to the port 8888
    let control    = ri2p::proto::stream::I2pStream::forwarded(8888).unwrap();
    let local_dest = control.get_local_dest().to_string();

    // spawn a thread for the client
    thread::spawn(move|| { client(local_dest) });

    // create a tcp socket that listens to the port 8888
    let listener = TcpListener::bind("127.0.0.1:8888").unwrap();
    let stream   = listener.accept().unwrap().0;

    let mut reader = BufReader::new(stream);
    let mut dest   = String::new();

    // the first line read from the socket
    // is always the destination of the connected remote peer
    reader.read_line(&mut dest).unwrap();

    println!("Accepted a stream! Remote peer: {}", dest);

    loop {
        let mut msg = String::new();
        reader.read_line(&mut msg).unwrap();
        print!("client sent: {}", msg);
    }
}
