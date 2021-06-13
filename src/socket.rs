use std::net::{TcpStream};
use std::io::{BufReader, BufWriter};

use crate::error::I2pError;

pub enum SocketType {
    Tcp,
    Udp,
}

struct UdpSocket {
}

struct TcpSocket {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

pub struct I2pSocket {
    stype: SocketType,
    tcp:   Option<TcpSocket>,
    udp:   Option<UdpSocket>,
}

fn udp_socket(host: &str, port: u16) -> Result<UdpSocket, I2pError> {
    todo!();
}

fn tcp_socket(host: &str, port: u16) -> Result<TcpSocket, I2pError> {

    let mut stream = match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(v)  => v,
        Err(e) => {
            eprintln!("Failed to connect to the router: {}", e);
            return Err(I2pError::TcpConnectionError);
        }
    };

    return Ok(TcpSocket {
        reader: BufReader::new(stream.try_clone().unwrap()),
        writer: BufWriter::new(stream),
    });
}

impl I2pSocket {

    pub fn new(stype: SocketType, host: &str, port: u16) -> Result<I2pSocket, I2pError> {
        match stype {
            SocketType::Tcp => {
                match tcp_socket(host, port) {
                    Ok(v) => return Ok(I2pSocket {
                        stype: SocketType::Tcp,
                        tcp:   Some(v),
                        udp:   None,
                    }),
                    Err(e) => return Err(e),
                };
            },
            SocketType::Udp => {
                match udp_socket(host, port) {
                    Ok(v) => return Ok(I2pSocket {
                        stype: SocketType::Udp,
                        tcp:   None,
                        udp:   Some(v),
                    }),
                    Err(e) => return Err(e),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp() {
        match I2pSocket::new(SocketType::Tcp, "localhost", 7656) {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("test_tcp: {:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_tcp_wrong_port() {
        match I2pSocket::new(SocketType::Tcp, "localhost", 7655) {
            Ok(_)  => assert!(false),
            Err(e) => assert!(true),
        }
    }
}
