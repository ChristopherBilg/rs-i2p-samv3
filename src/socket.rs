use std::net::{TcpStream, UdpSocket};
use std::io::{BufReader, BufWriter, Write, BufRead, Read};
use std::time::Duration;

use crate::error::I2pError;
use crate::cmd::hello;

const SAM_TCP_PORT: u16  = 7656;
const SAM_UDP_PORT: u16  = 7655;

pub struct I2pDatagramSocket {
    socket: UdpSocket,
}

pub struct I2pStreamSocket {
    writer: BufWriter<TcpStream>,
    reader: BufReader<TcpStream>,
}

pub trait I2pControlSocket: Sized {
    fn read_cmd(&mut self, buf: &mut String) -> Result<usize, I2pError>;
    fn write_cmd(&mut self, buf: &String) -> Result<(), I2pError>;
    fn write(&mut self, buf: &[u8]) -> Result<(), I2pError>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, I2pError>;
}

fn udp_socket(host: &str, port: u16) -> Result<I2pDatagramSocket, I2pError> {

    let socket = match UdpSocket::bind(format!("{}:{}", host, port)) {
        Ok(v)  => v,
        Err(e) => {
            eprintln!("Failed to connect to the router: {}", e);
            return Err(I2pError::TcpConnectionError);
        }
    };

    match socket.set_read_timeout(Some(Duration::from_millis(60 * 1000))) {
        Ok(_)  => {},
        Err(e) => {
            eprintln!("Failed to set timeout for read operation: {}", e);
            return Err(I2pError::Unknown);
        }
    }

    return Ok(I2pDatagramSocket {
        socket: socket,
    });
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

impl I2pDatagramSocket {

    pub fn new(port: u16) -> Result<Self, I2pError> {
        match udp_socket("127.0.0.1", port) {
            Ok(v)  => Ok(v),
            Err(e) => Err(e),
        }
    }
}

impl I2pStreamSocket {

    pub fn new() -> Result<I2pStreamSocket, I2pError> {
        match tcp_socket("127.0.0.1", SAM_TCP_PORT) {
            Ok(v)  => Ok(v),
            Err(e) => Err(e),
        }
    }

    pub fn connected() -> Result<I2pStreamSocket, I2pError> {
        let mut socket = match tcp_socket("127.0.0.1", SAM_TCP_PORT) {
            Ok(v)  => v,
            Err(e) => return Err(e),
        };

        match hello::handshake(&mut socket) {
            Ok(_)  => { Ok(socket) },
            Err(e) => return Err(e),
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

impl I2pControlSocket for I2pDatagramSocket{

    fn read_cmd(&mut self, buf: &mut String) -> Result<usize, I2pError> {
        let bytes = unsafe { buf.as_bytes_mut() };

        match self.socket.recv(bytes) {
            Ok(nread) => {
                if nread == 0 {
                    eprintln!("Received 0 bytes from the router");
                    return Err(I2pError::UdpReadError);
                }
                return Ok(nread);
            }
            Err(e) => {
                eprintln!("Failed to receive UDP data: {}", e);
                return Err(I2pError::UdpReadError);
            }
        }
    }

    fn write_cmd(&mut self, buf: &String) -> Result<(), I2pError> {
        // TODO verify message

        match self.socket.send_to(buf.as_bytes(), format!("127.0.0.1:{}", SAM_UDP_PORT)) {
            Ok(_)  => Ok(()),
            Err(e) => {
                eprintln!("Failed to send UDP data: {}", e);
                return Err(I2pError::UdpWriteError);
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), I2pError> {
        match self.socket.send_to(buf, format!("127.0.0.1:{}", SAM_UDP_PORT)) {
            Ok(_)  => Ok(()),
            Err(e) => {
                eprintln!("Failed to send UDP data: {}", e);
                return Err(I2pError::UdpWriteError);
            }
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, I2pError> {
        match self.socket.recv(buf) {
            Ok(nread) => {
                if nread == 0 {
                    eprintln!("Received 0 bytes from the router");
                    return Err(I2pError::UdpReadError);
                }
                return Ok(nread);
            }
            Err(e) => {
                eprintln!("Failed to receive UDP data: {}", e);
                return Err(I2pError::UdpReadError);
            }
        }
    }
}

impl I2pControlSocket for I2pStreamSocket {

    /// See documentation for BufReader::read_line()
    fn read_cmd(&mut self, buf: &mut String) -> Result<usize, I2pError> {
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

    /// See documentation for Write::write()
    ///
    /// # Notes
    /// - `buf` must not be empty
    /// - `buf` must end in \n
    ///
    fn write_cmd(&mut self, buf: &String) -> Result<(), I2pError> {
        if buf.len() == 0 {
            return Err(I2pError::InvalidValue);
        }

        // TODO verify that last byte of String is \n

        match self.writer.write(buf.as_bytes()) {
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

    /// See documentation for Write::write()
    ///
    /// # Notes
    /// - `buf` must not be empty
    ///
    fn write(&mut self, buf: &[u8]) -> Result<(), I2pError> {
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

    /// See documentation for Read::read()
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, I2pError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_new() {
        match I2pStreamSocket::connected() {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("test_tcp: {:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_tcp_connected() {
        match I2pStreamSocket::connected() {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("test_tcp: {:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_tcp_wrong_port() {
        match tcp_socket("127.0.0.1", 7655) {
            Ok(_)  => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_tcp_send() {
        match I2pStreamSocket::connected() {
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
        match I2pStreamSocket::connected() {
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
