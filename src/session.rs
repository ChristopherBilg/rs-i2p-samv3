use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use crate::error::I2pError;
use crate::socket::I2pStreamSocket;
use crate::cmd::*;

pub enum SessionType {
    VirtualStream,
    RepliableDatagram,
    AnonymousDatagram,
}

pub struct I2pSession {
    pub socket: I2pStreamSocket,
    pub nick:   String,
    pub local:  String,
}

impl I2pSession {

    /// Start a new session with the I2P router
    ///
    /// Connect to the router via the default SAM gateway (localhost:7656)
    /// and create a control socket, which is used to create the actual session,
    /// and a nickname for the client (random alphanumeric string)
    ///
    /// # Arguments
    ///
    /// `stype` - Session type: Virtual stream, Repliable or Anonymous datagram
    ///
    pub fn new(stype: SessionType) -> Result<I2pSession, I2pError> {

        let mut socket = match I2pStreamSocket::connected() {
            Ok(v)  => v,
            Err(e) => {
                eprintln!("Failed to connect to the router: {:#?}", e);
                return Err(I2pError::TcpConnectionError);
            }
        };

        // generate random nickname
        let nick: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        // create a new session of type "stype"
        match session::create(&mut socket, &stype, &nick) {
            Ok(_)  => {},
            Err(e) => return Err(e),
        }

        // and fetch our local destination
        let dest = match naming::lookup(&mut socket, "ME") {
            Ok(v) => {
                if v.1 == "" {
                    return Err(I2pError::InvalidValue);
                }
                v.1
            },
            Err(e) => return Err(e),
        };

        Ok(I2pSession {
            socket: socket,
            nick:   nick.to_string(),
            local:  dest.to_string(),
        })
    }

    pub fn new_socket(stype: SessionType, port: u16) -> Result<I2pSession, I2pError> {

        let mut socket = match I2pStreamSocket::connected() {
            Ok(v)  => v,
            Err(e) => {
                eprintln!("Failed to connect to the router: {:#?}", e);
                return Err(I2pError::TcpConnectionError);
            }
        };

        // generate random nickname
        let nick: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();

        // create a new session of type "stype"
        match session::create_dgram(&mut socket, &stype, &nick, port) {
            Ok(_)  => {},
            Err(e) => return Err(e),
        }

        // and fetch our local destination
        let dest = match naming::lookup(&mut socket, "ME") {
            Ok(v) => {
                if v.1 == "" {
                    return Err(I2pError::InvalidValue);
                }
                v.1
            },
            Err(e) => return Err(e),
        };

        Ok(I2pSession {
            socket: socket,
            nick:   nick.to_string(),
            local:  dest.to_string(),
        })
    }


    /// TODO
    pub fn destroy(&self) -> Result<(), I2pError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_create_stream() {
        match I2pSession::new(SessionType::VirtualStream) {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("{:#?}", e);
                assert!(false);
            }
        }

    }

    #[test]
    fn test_session_create_raw() {
        match I2pSession::new(SessionType::AnonymousDatagram) {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("{:#?}", e);
                assert!(false);
            }
        }
    }
}
