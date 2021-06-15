use std::net::{TcpStream};
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::time::Duration;

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

impl I2pSocket {

    pub fn new(stype: SocketType, host: &str, port: u16) -> Result<I2pSocket, I2pError> {
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
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), I2pError> {
        if buf.len() == 0 {
            return Err(I2pError::InvalidValue);
        }

        match self.stype {
            SocketType::Tcp => {
                return self.tcp_write(buf);
            },
            SocketType::Udp => {
                return self.udp_write(buf);
            }
        }
    }

    fn tcp_write(&mut self, buf: &[u8]) -> Result<(), I2pError> {
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

    fn udp_write(&self, _buf: &[u8]) -> Result<(), I2pError> {
        todo!();
    }

    pub fn read(&mut self, buf: &mut String) -> Result<(), I2pError> {
        match self.stype {
            SocketType::Tcp => {
                return self.tcp_read(buf);
            },
            SocketType::Udp => {
                return self.udp_read(buf);
            }
        }
    }

    fn tcp_read(&mut self, buf: &mut String) -> Result<(), I2pError> {
        match &mut self.tcp {
            Some(tcp) => {
                match tcp.reader.read_line(buf) {
                    Ok(nread)  => {
                        if nread == 0 {
                            eprintln!("Received 0 bytes from the router");
                            return Err(I2pError::TcpStreamError);
                        }
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

    fn udp_read(&self, _buf: &mut String) -> Result<(), I2pError> {
        todo!();
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
                // TODO try to send empty buffer
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
                // TODO try to send empty buffer
            },
            Err(_) => {
                assert!(false);
            }
        }
    }
}
