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

    /// Create new I2P session for a virtual stream
    pub fn stream() -> Result<I2pSession, I2pError> {

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

        // create a new virtual stream session
        match session::stream(&mut socket, &nick) {
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

    /// Create a new I2P session for an anonymous/repliable datagram
    pub fn datagram(stype: SessionType, port: u16) -> Result<I2pSession, I2pError> {

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
        match session::datagram(&mut socket, &stype, &nick, port) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_create_stream() {
        match I2pSession::stream() {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("{:#?}", e);
                assert!(false);
            }
        }

    }

    #[test]
    fn test_session_create_raw() {
        match I2pSession::datagram(SessionType::AnonymousDatagram, 8888) {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("{:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_session_create_repliable() {
        match I2pSession::datagram(SessionType::RepliableDatagram, 9999) {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("{:#?}", e);
                assert!(false);
            }
        }
    }
}
