use crate::error::I2pError;
use crate::socket::I2pSocket;
use crate::parser::{Command, Subcommand, parse};
use crate::cmd::aux;

fn parser(response: &str) -> Result<Vec<(String, String)>, I2pError> {

    let parsed = match parse(response, Command::Stream, Some(Subcommand::Status)) {
        Ok(v)  => v,
        Err(e) => {
            eprintln!("Failed to parse response: {:#?}", e);
            return Err(I2pError::InvalidValue);
        }
    };

    match parsed.get_value("RESULT") {
        Some(v)  => {
            match &v[..] {
                "OK" => {
                    return Ok(Vec::new());
                },
                _ => {
                    eprintln!("Invalid response from router: {}", v);
                    return Err(I2pError::InvalidValue);
                }
            }
        },
        None => {
            eprintln!("Router respones did not contain RESULT!");
            return Err(I2pError::InvalidValue);
        }
    }
}

/// Connect to a remote peer using a destination address
///
/// # Arguments
///
/// `socket` - I2pSocket object created by the caller
/// `nick` - Nickname of the client, generated during I2pSession creation
/// `host` - Destination address of the remote peer (normal or a b32 address, or a public key)
///
pub fn connect(socket: &mut I2pSocket, nick: &str, host: &str) -> Result<(), I2pError> {
    let msg = format!("STREAM CONNECT ID={} DESTINATION={} SILENT=false\n", nick, host);

    match aux::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

/// Accept connection from a remote peer
///
/// # Arguments
///
/// `socket` - I2pSocket object created by the caller
/// `nick` - Nickname of the client, generated during I2pSession creation
///
pub fn accept(socket: &mut I2pSocket, nick: &str) -> Result<(), I2pError> {
    let msg = format!("STREAM ACCEPT ID={} SILENT=false\n", nick);

    match aux::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{I2pSession, SessionType};
    use crate::socket::{I2pSocket, SocketType};
    use crate::proto::stream::I2pStream;
    use std::thread;
    use std::time;

    #[test]
    fn test_connection() {
        let session = I2pSession::new(SessionType::VirtualStream).unwrap();
        let mut socket = I2pSocket::new(SocketType::Tcp, "localhost", 7656).unwrap();

        // valid nickname and host
        assert_eq!(
            connect(&mut socket, &session.nick, "idk.i2p"),
            Ok(()),
        );

        // invalid nickname
        assert_eq!(
            connect(&mut socket, "invalid_nick", "idk.i2p"),
            Err(I2pError::InvalidValue),
        );

        // invalid host
        assert_eq!(
            connect(&mut socket, &session.nick, "zkzkk3k3kkfksfsdf.com"),
            Err(I2pError::InvalidValue),
        );
    }

    #[test]
    fn test_accept_invalid_nick() {
        let session    = I2pSession::new(SessionType::VirtualStream).unwrap();
        let mut socket = I2pSocket::new(SocketType::Tcp, "localhost", 7656).unwrap();

        // invalid nickname
        assert_eq!(
            accept(&mut socket, "invalid_nick"),
            Err(I2pError::InvalidValue),
        );
    }

    #[test]
    fn test_accept_server() {
        let session    = I2pSession::new(SessionType::VirtualStream).unwrap();
        let mut socket = I2pSocket::new(SocketType::Tcp, "localhost", 7656).unwrap();
        let local_dest = session.nick.clone();

        // spawn a thread for the client and notify the router
        // that we're readyt to accept a peer connection
        thread::spawn(move|| { client(local_dest) });

        assert_eq!(
            accept(&mut socket, &session.nick),
            Ok(()),
        );
    }

    fn client(dest: String) {
        std::thread::sleep(time::Duration::from_millis(2000));
        let mut stream = I2pStream::new().unwrap();

        stream.connect(&dest).unwrap();
    }
}
