use crate::session::*;
use crate::error::I2pError;
use crate::socket::{I2pStreamSocket, SocketType};
use crate::cmd::*;

pub struct I2pStream {
    _session: I2pSession,
    socket:   I2pStreamSocket,
}

impl I2pStream {

    pub fn new() -> Result<I2pStream, I2pError> {
        let session = match I2pSession::new(SessionType::VirtualStream) {
            Ok(v)  => v,
            Err(e) => return Err(e),
        };

        // VirtualStream session was created successfully, now create actual client socket
        let socket = match I2pStreamSocket::new() {
            Ok(v)  => v,
            Err(e) => {
                eprintln!("Failed to connect to the router: {:#?}", e);
                return Err(I2pError::TcpConnectionError);
            }
        };

        Ok(I2pStream {
            _session: session,
            socket:  socket,
        })
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
        match stream::connect(&mut self.socket, &self._session.nick, &addr) {
            Ok(_)  => { },
            Err(e) => return Err(e),
        }

        Ok(())
    }

    /// Accept a virtual stream connection from an I2P peer
    ///
    /// Function returns Ok(()) when a remote connection has been accepted
    /// and an I2pError if there was an issue with the router.
    ///
    pub fn accept(&mut self) -> Result<(), I2pError> {
        // router sent as RESULT=OK
        match stream::accept(&mut self.socket, &self._session.nick) {
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

    pub fn get_local_dest(&self) -> &str {
        return &self._session.local;
    }

    pub fn get_nick(&self) -> &str {
        return &self._session.nick;
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
