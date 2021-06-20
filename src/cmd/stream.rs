use crate::error::I2pError;
use crate::socket::I2pStreamSocket;
use crate::parser::{Command, Subcommand, parse};
use crate::cmd::helper;

fn parser(response: &str) -> Result<Vec<(String, String)>, I2pError> {

    let parsed = match parse(response, Command::Stream, Some(Subcommand::Status)) {
        Ok(v)  => v,
        Err(e) => {
            eprintln!("Failed to parse response: {:#?}", e);
            return Err(I2pError::InvalidValue);
        }
    };

    match helper::check_result(&parsed) {
        Ok(_) => {
            Ok(Vec::new())
        },
        Err(e) => {
            eprintln!("Response did not contain RESULT=OK: {:#?}", e.0);
            eprintln!("Message: {}", e.1);
            Err(e.0)
        }
    }
}

/// Connect to a remote peer using a destination address
///
/// # Arguments
///
/// `socket` - I2pStreamSocket object created by the caller
/// `nick` - Nickname of the client, generated during I2pSession creation
/// `host` - Destination address of the remote peer (normal or a b32 address, or a public key)
///
pub fn connect(socket: &mut I2pStreamSocket, nick: &str, host: &str) -> Result<(), I2pError> {
    let msg = format!("STREAM CONNECT ID={} DESTINATION={} SILENT=false\n", nick, host);

    match helper::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

/// Accept connection from a remote peer
///
/// # Arguments
///
/// `socket` - I2pStreamSocket object created by the caller
/// `nick` - Nickname of the client, generated during I2pSession creation
///
pub fn accept(socket: &mut I2pStreamSocket, nick: &str) -> Result<(), I2pError> {
    let msg = format!("STREAM ACCEPT ID={} SILENT=false\n", nick);

    match helper::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

/// Notify that the router that an incoming virtual stream should use
/// a local TCP listener instead of this socket
///
/// # Arguments
/// `socket` - I2pStreamSocket object created by the caller
/// `nick` - Nickname of the client, generated during I2pSession creation
/// `port` - Port that the local TCP listener is listening to
///
pub fn forward(socket: &mut I2pStreamSocket, nick: &str, port: u16) -> Result<(), I2pError> {
    let msg = format!("STREAM FORWARD ID={} PORT={} SILENT=false\n", nick, port);

    match helper::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::I2pSession;
    use crate::socket::I2pStreamSocket;

    #[test]
    fn test_cmd_stream_connection() {
        let session = I2pSession::stream().unwrap();
        let mut socket = I2pStreamSocket::connected().unwrap();

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
        let mut socket = I2pStreamSocket::connected().unwrap();

        // invalid nickname
        assert_eq!(
            accept(&mut socket, "invalid_nick"),
            Err(I2pError::InvalidValue),
        );
    }

    #[test]
    fn test_cmd_stream_accept_server() {
        let session    = I2pSession::stream().unwrap();
        let mut socket = I2pStreamSocket::connected().unwrap();

        assert_eq!(
            accept(&mut socket, &session.nick),
            Ok(()),
        );
    }

    #[test]
    fn test_cmd_stream_forward_server() {
        let session    = I2pSession::stream().unwrap();
        let mut socket = I2pStreamSocket::connected().unwrap();

        assert_eq!(
            forward(&mut socket, &session.nick, 8888),
            Ok(()),
        );
    }
}
