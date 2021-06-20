use crate::session::*;
use crate::error::I2pError;
use crate::socket::{I2pControlSocket, I2pStreamSocket};
use crate::cmd::*;

pub struct I2pStream {
    session: I2pSession,
    socket:  I2pStreamSocket,
}

fn new() -> Result<I2pStream, I2pError> {
    let session = match I2pSession::stream() {
        Ok(v)  => v,
        Err(e) => return Err(e),
    };

    // VirtualStream session was created successfully, now create actual client socket
    let socket = match I2pStreamSocket::connected() {
        Ok(v)  => v,
        Err(e) => {
            eprintln!("Failed to connect to the router: {:#?}", e);
            return Err(I2pError::TcpConnectionError);
        }
    };

    Ok(I2pStream {
        session: session,
        socket:  socket,
    })
}

impl I2pStream {

    /// Create a new I2P virtual stream object
    /// 
    /// If the call succeeds, the returned stream object is not yet
    /// connected to anything, and does not expect any incoming requests
    /// so either connect() or accept must be called
    pub fn new() -> Result<I2pStream, I2pError> {
        new()
    }

    /// Establish a virtual stream connection to an I2P host
    ///
    /// Function returns Ok(()) when the connection has been established
    /// and an I2pError if the connection failed (e.g, peer is offline)
    ///
    /// # Arguments
    /// `addr` - an I2P address (normal or b32), or a public key of remote peer
    ///
    pub fn connect(&mut self, addr: &str) -> Result<(), I2pError> {
        match stream::connect(&mut self.socket, &self.session.nick, &addr) {
            Ok(_)  => { },
            Err(e) => return Err(e),
        }

        Ok(())
    }

    /// Create a new session for a forwarded I2P virtual stream
    ///
    /// The stream object that is returned is not used for data
    /// exchange but instead a new TCP listener must be created
    /// that listens to the port provided.
    ///
    /// If this function succeeds, I2P router tries to connect
    /// to that socket and when a remote peer wants to connect
    /// to the socket, the router will route all data coming
    /// from the remote peer to the socket
    ///
    /// # Arguments
    /// `port` - port of the new TCP server that router should connect to
    ///
    pub fn forwarded(port: u16) -> Result<I2pStream, I2pError> {
        let mut stream = match new() {
            Ok(v)  => v,
            Err(e) => return Err(e),
        };

        match stream::forward(&mut stream.socket, &stream.session.nick, port) {
            Ok(_)  => { },
            Err(e) => return Err(e),
        }

        Ok(stream)
    }

    /// Accept a virtual stream connection from an I2P peer
    ///
    /// Function returns Ok(()) when a remote connection has been accepted
    /// and an I2pError if there was an issue with the router.
    ///
    pub fn accept(&mut self) -> Result<(), I2pError> {
        match stream::accept(&mut self.socket, &self.session.nick) {
            Ok(_)  => { },
            Err(e) => return Err(e),
        }

        // wait until a peer connects and then return the socket to the user
        let mut peer = String::new();

        match &mut self.socket.read_line(&mut peer) {
            Ok(_) => Ok(()),
            Err(_) => {
                return Err(I2pError::RouterError);
            }
        }
    }

    /// Get the local destination of peer
    pub fn get_local_dest(&self) -> &str {
        return &self.session.local;
    }

    /// Get the assigned random nickname of peer
    pub fn get_nick(&self) -> &str {
        return &self.session.nick;
    }

    /// Write data to the I2P socket
    ///
    /// Internally this function calls Write::write()
    pub fn write(&mut self, buf: &[u8]) -> Result<(), I2pError> {
        self.socket.write(buf)
    }

    /// Read data from the I2P socket
    ///
    /// Internally this function calls Read::read()
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, I2pError> {
        self.socket.read(buf)
    }

    /// Read data from the I2P socket
    ///
    /// Internally this function calls Read::read_to_string()
    pub fn read_to_string(&mut self, buf: &mut String) -> Result<usize, I2pError> {
        self.socket.read_to_string(buf)
    }

    /// Read data from the I2P socket
    ///
    /// Internally this function calls Read::read_exact()
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), I2pError> {
        self.socket.read_exact(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::stream::I2pStream;

    #[test]
    fn test_stream_new() {
        match I2pStream::new() {
            Ok(v) => {
                assert!(true);
            },
            Err(e) => {
                eprintln!("test_stream_new: {:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_stream_connect_valid() {
        let mut stream = I2pStream::new().unwrap();

        match stream.connect("idk.i2p") {
            Ok(_) => {
                assert!(true);
            },
            Err(e) => {
                eprintln!("test_stream_connect_valid: {:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_stream_connect_invalid() {
        let mut stream = I2pStream::new().unwrap();

        match stream.connect("google.com") {
            Ok(_) => {
                assert!(false);
            },
            Err(e) => {
                assert!(true);
            }
        }

    }
}
