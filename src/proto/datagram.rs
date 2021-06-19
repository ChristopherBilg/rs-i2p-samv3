use crate::session::*;
use crate::error::I2pError;
use crate::socket::{I2pSocket, I2pStreamSocket, I2pDatagramSocket, Streamable};
use crate::cmd::*;
use crate::parser;

pub struct I2pRawSocket {
    _session: I2pSession,
    socket:   I2pDatagramSocket,
}

pub struct I2pRepliableSocket {
    _session: I2pSession,
    socket:   I2pDatagramSocket,
    buffer:   Vec<u8>,
}

impl I2pRawSocket {

    pub fn new(port: u16) -> Result<I2pRawSocket, I2pError> {
        let session = match I2pSession::new_socket(SessionType::AnonymousDatagram, port) {
            Ok(v)  => v,
            Err(e) => return Err(e),
        };

        // Session was created successfully, now create actual client socket
        let socket = match I2pDatagramSocket::new_sock(port) {
            Ok(v)  => v,
            Err(e) => {
                eprintln!("Failed to connect to the router: {:#?}", e);
                return Err(I2pError::TcpConnectionError);
            }
        };

        Ok(I2pRawSocket {
            _session: session,
            socket:  socket,
        })
    }

    /// Get the destination of this session
    pub fn get_local_dest(&self) -> &str {
        return &self._session.local;
    }

    /// Get the nickname assigned to this session
    pub fn get_nick(&self) -> &str {
        return &self._session.nick;
    }

    /// Write data to the I2P socket
    pub fn send_to(&mut self, buf: &[u8], dest: &str) -> Result<(), I2pError> {
        let mut header = format!("3.0 {} {}\n", self._session.nick, dest)
            .as_bytes()
            .to_vec();
        header.extend_from_slice(buf);
        self.socket.write(&header)
    }

    /// Read data from the I2P socket
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, I2pError> {
        self.socket.read(buf)
    }
}

impl I2pRepliableSocket {

    pub fn new(port: u16) -> Result<I2pRepliableSocket, I2pError> {
        let session = match I2pSession::new_socket(SessionType::RepliableDatagram, port) {
            Ok(v)  => v,
            Err(e) => return Err(e),
        };

        // Session was created successfully, now create actual client socket
        let socket = match I2pDatagramSocket::new_sock(port) {
            Ok(v)  => v,
            Err(e) => {
                eprintln!("Failed to connect to the router: {:#?}", e);
                return Err(I2pError::TcpConnectionError);
            }
        };

        Ok(I2pRepliableSocket {
            _session: session,
            socket:   socket,
            buffer:   vec![0; 65536],
        })
    }

    /// Get the destination of this session
    pub fn get_local_dest(&self) -> &str {
        return &self._session.local;
    }

    /// Get the nickname assigned to this session
    pub fn get_nick(&self) -> &str {
        return &self._session.nick;
    }

    /// Write data to the I2P socket
    pub fn send_to(&mut self, buf: &[u8], dest: &str) -> Result<(), I2pError> {
        let mut header = format!("3.0 {} {}\n", self._session.nick, dest)
            .as_bytes()
            .to_vec();
        header.extend_from_slice(buf);
        self.socket.write(&header)
    }

    /// Read data from the I2P socket
    pub fn recv_from(&mut self, buf: &mut [u8]) -> Result<(usize, String), I2pError> {
        match self.socket.read(&mut self.buffer) {
            Ok(nread) => {
                match std::str::from_utf8(&self.buffer) {
                    Ok(data) => {
                        match parser::parse_header(&data) {
                            Ok(parsed) => {
                                for (place, data) in buf.iter_mut().zip(parsed.1.as_bytes().iter()) {
                                    *place = *data
                                }
                                return Ok((nread - parsed.0.dest.len(), parsed.0.dest.to_string()));
                            },
                            Err(e) => {
                                eprintln!("Failed to parse repliable datagram: {:#?}", e);
                                return Err(e);
                            }
                        }
                    },
                    Err(e) => {
                        println!("Failed to convert data to u8 for parser");
                        return Err(I2pError::InvalidValue);
                    }
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    /// Read data from I2P socket
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, I2pError> {
        match self.recv_from(buf) {
            Ok(v) => {
                Ok(v.0)
            },
            Err(e) => {
                eprintln!("{:#?}", e);
                return Err(e);
            }
        }
    }
}
