use std::net::{TcpStream};
use std::io::{BufReader, BufWriter, Write, BufRead, Read};
use std::time::Duration;

use crate::error::I2pError;
use crate::cmd::hello;

const SAM_TCP_PORT: u16  = 7656;
const _SAM_UDP_PORT: u16 = 7655;

pub enum SocketType {
    Tcp,
    Udp,
}

struct UdpSocket {
}

pub struct I2pStreamSocket {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

fn udp_socket(_host: &str, _port: u16) -> Result<UdpSocket, I2pError> {
    todo!();
}

fn tcp_socket(host: &str, port: u16) -> Result<I2pStreamSocket, I2pError> {

    let stream = match TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(v)  => v,
        Err(e) => {
            eprintln!("Failed to connect to the router: {}", e);
            return Err(I2pError::TcpConnectionError);
        }
    };

    match stream.set_read_timeout(Some(Duration::from_millis(2 * 60 * 1000))) {
        Ok(_)  => {},
        Err(e) => {
            eprintln!("Failed to set timeout for read operation: {}", e);
            return Err(I2pError::Unknown);
        }
    }

    return Ok(I2pStreamSocket {
        reader: BufReader::new(stream.try_clone().unwrap()),
        writer: BufWriter::new(stream),
    });
}

impl I2pStreamSocket {

    pub fn new() -> Result<I2pStreamSocket, I2pError> {
        let mut socket = match tcp_socket("localhost", SAM_TCP_PORT) {
            Ok(v)  => v,
            Err(e) => return Err(e),
        };

        match hello::handshake(&mut socket) {
            Ok(_)  => { Ok(socket) },
            Err(e) => return Err(e),
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), I2pError> {
        if buf.len() == 0 {
            return Err(I2pError::InvalidValue);
        }

        match self.writer.write(buf) {
            Ok(_)  => {
                self.writer.flush().unwrap();
                return Ok(());
            },
            Err(e) => {
                eprintln!("Failed to send TCP data: {}", e);
                return Err(I2pError::TcpStreamError);
            }
        }
    }

    /// See documentation for BufReader::read()
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, I2pError> {
        match self.reader.read(buf) {
            Ok(nread)  => {
                if nread == 0 {
                    eprintln!("Received 0 bytes from the router");
                    return Err(I2pError::TcpStreamError);
                }
                return Ok(nread);
            }
            Err(e) => {
                eprintln!("Failed to receive TCP data: {}", e);
                return Err(I2pError::TcpStreamError);
            }
        }
    }

    /// See documentation for BufReader::read_line()
    pub fn read_line(&mut self, buf: &mut String) -> Result<usize, I2pError> {
        match self.reader.read_line(buf) {
            Ok(nread)  => {
                if nread == 0 {
                    eprintln!("Received 0 bytes from the router");
                    return Err(I2pError::TcpStreamError);
                }
                return Ok(nread);
            }
            Err(e) => {
                eprintln!("Failed to receive TCP data: {}", e);
                return Err(I2pError::TcpStreamError);
            }
        }
    }

    /// See documentation for Read::read_to_string()
    pub fn read_to_string(&mut self, buf: &mut String) -> Result<usize, I2pError> {
        match self.reader.read_to_string(buf) {
            Ok(nread)  => {
                if nread == 0 {
                    eprintln!("Received 0 bytes from the router");
                    return Err(I2pError::TcpStreamError);
                }
                return Ok(nread);
            }
            Err(e) => {
                eprintln!("Failed to receive TCP data: {}", e);
                return Err(I2pError::TcpStreamError);
            }
        }
    }

    /// See documentation for Read::read_exact()
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), I2pError> {
        match self.reader.read_exact(buf) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                eprintln!("Failed to receive TCP data: {}", e);
                return Err(I2pError::TcpStreamError);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp() {
        match I2pStreamSocket::new() {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("test_tcp: {:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    #[ignore]
    fn test_tcp_wrong_port() {
        match I2pStreamSocket::new() {
            Ok(_)  => assert!(false),
            Err(_) => assert!(true),
        }
    }

    /*
    #[test]
    fn test_tcp_uninit() {
        match new_uninit() {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("test_tcp: {:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_tcp_wrong_port_uninit() {
        match new_uninit() {
            Ok(_)  => assert!(false),
            Err(_) => assert!(true),
        }
    }
    */

    #[test]
    fn test_tcp_send() {
        match I2pStreamSocket::new() {
            Ok(mut socket)  => {
                match socket.write("PING".as_bytes()) {
                    Ok(_)  => assert!(true),
                    Err(e) => {
                        eprintln!("test_tcp_send: {:#?}", e);
                        assert!(false);
                    }
                }
            },
            Err(_) => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_tcp_send_empty() {
        match I2pStreamSocket::new() {
            Ok(mut socket)  => {
                let vec: Vec<u8> = Vec::new();

                match socket.write(&vec) {
                    Ok(_)  => assert!(false),
                    Err(_) => assert!(true),
                }
            },
            Err(_) => {
                assert!(false);
            }
        }
    }
}
