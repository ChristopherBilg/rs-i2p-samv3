use std::env;
use std::thread;
use std::time;

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

    // establish a connection with the router by creating a new I2pStream object
    let mut stream = ri2p::proto::stream::I2pStream::new().unwrap();
    let local_dest = stream.get_local_dest().to_string();

    // spawn a thread for the client
    thread::spawn(move|| { client(local_dest) });

    // notify the router that we're ready to accept a connection
    println!("waiting for an incoming connection...");
    stream.accept().unwrap();

    loop {
        let mut buf = vec![0; 14];

        match stream.read_exact(&mut buf) {
            Ok(_) => {
                println!("client sent: '{:#?}'", std::str::from_utf8(&buf).unwrap());
            },
            Err(e) => {
                eprintln!("failure: {:#?}", e);
            }
        }
    }
}
