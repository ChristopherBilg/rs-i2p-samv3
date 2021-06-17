use std::net::{TcpStream};
use std::io::{BufReader, BufWriter, Write, BufRead, Read};
use std::time::Duration;

use crate::error::I2pError;
use crate::cmd::hello;

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
    _udp:  Option<UdpSocket>,
}

fn udp_socket(_host: &str, _port: u16) -> Result<UdpSocket, I2pError> {
    todo!();
}

fn tcp_socket(host: &str, port: u16) -> Result<TcpSocket, I2pError> {

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

    return Ok(TcpSocket {
        reader: BufReader::new(stream.try_clone().unwrap()),
        writer: BufWriter::new(stream),
    });
}

/// Create new, uninitialized socket
///
/// Create a socket but do not handshake with the router to establish the initial connection
///
/// # Arguments
///
/// `stype` - Socket type (virtual stream, anonymous or repliable datagram)
/// `host` - SAM host server address
/// `port`- SAM host server port
///
pub fn new_uninit(stype: SocketType, host: &str, port: u16) -> Result<I2pSocket, I2pError> {
    match stype {
        SocketType::Tcp => {
            match tcp_socket(host, port) {
                Ok(v) => return Ok(I2pSocket {
                    stype: SocketType::Tcp,
                    tcp:   Some(v),
                    _udp:  None,
                }),
                Err(e) => return Err(e),
            };
        },
        SocketType::Udp => {
            match udp_socket(host, port) {
                Ok(v) => return Ok(I2pSocket {
                    stype: SocketType::Udp,
                    tcp:   None,
                    _udp:  Some(v),
                }),
                Err(e) => return Err(e),
            };
        }
    };
}

impl I2pSocket {

    pub fn new(stype: SocketType, host: &str, port: u16) -> Result<I2pSocket, I2pError> {
        let mut socket = match new_uninit(stype, host, port) {
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

        match &mut self.tcp {
            Some(tcp) => {
                match tcp.writer.write(buf) {
                    Ok(_)  => {
                        tcp.writer.flush().unwrap();
                        return Ok(());
                    },
                    Err(e) => {
                        eprintln!("Failed to send TCP data: {}", e);
                        return Err(I2pError::TcpStreamError);
                    }
                }
            },
            None => {
                panic!();
            }
        }
    }

    /// See documentation for BufReader::read()
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, I2pError> {
        match &mut self.tcp {
            Some(tcp) => {
                match tcp.reader.read(buf) {
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
            },
            None => {
                panic!();
            }
        }
    }

    /// See documentation for BufReader::read_line()
    pub fn read_line(&mut self, buf: &mut String) -> Result<usize, I2pError> {
        match &mut self.tcp {
            Some(tcp) => {
                match tcp.reader.read_line(buf) {
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
            },
            None => {
                panic!();
            }
        }
    }

    /// See documentation for Read::read_to_string()
    pub fn read_to_string(&mut self, buf: &mut String) -> Result<usize, I2pError> {
        match &mut self.tcp {
            Some(tcp) => {
                match tcp.reader.read_to_string(buf) {
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
            },
            None => {
                panic!();
            }
        }
    }

    /// See documentation for Read::read_exact()
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), I2pError> {
        match &mut self.tcp {
            Some(tcp) => {
                match tcp.reader.read_exact(buf) {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Failed to receive TCP data: {}", e);
                        return Err(I2pError::TcpStreamError);
                    }
                }
            },
            None => {
                panic!();
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
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_tcp_uninit() {
        match new_uninit(SocketType::Tcp, "localhost", 7656) {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("test_tcp: {:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_tcp_wrong_port_uninit() {
        match new_uninit(SocketType::Tcp, "localhost", 7655) {
            Ok(_)  => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_tcp_send() {
        match I2pSocket::new(SocketType::Tcp, "localhost", 7656) {
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
        match I2pSocket::new(SocketType::Tcp, "localhost", 7656) {
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
