use std::env;

extern crate ri2p;

fn main() {

    let args: Vec<String> = env::args().collect();

    let mut stream = ri2p::proto::stream::I2pStream::new().unwrap();

    stream.connect(&args[1]).unwrap();
    stream.write("GET / HTTP/1.1\r\n\n".as_bytes()).unwrap();

    let mut resp = String::new();
    stream.read_to_string(&mut resp).unwrap();

    println!("{}", resp);
}
