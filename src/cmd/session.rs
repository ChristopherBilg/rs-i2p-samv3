use crate::error::I2pError;
use crate::socket::I2pStreamSocket;
use crate::parser::{Command, Subcommand, parse};
use crate::session::SessionType;
use crate::cmd::helper;

/// Parse and validate router's SAMv3-compatible response
///
/// If the message is valid, return the parsed Message object to caller
///
/// # Arguments
/// `response` - Router's response in text format
///
fn parser(response: &str) -> Result<Vec<(String, String)>, I2pError> {

    let parsed = match parse(response, Command::Session, Some(Subcommand::Status)) {
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

pub fn datagram(
    socket: &mut I2pStreamSocket,
    stype:  &SessionType,
    nick:   &str,
    port:   u16)
    -> Result<(), I2pError>
{
    let msg = match stype {
        SessionType::RepliableDatagram => {
            format!("SESSION CREATE STYLE=DATAGRAM ID={} PORT={} DESTINATION=TRANSIENT\n",
                    nick, port)
        },
        SessionType::AnonymousDatagram => {
            format!("SESSION CREATE STYLE=RAW ID={} PORT={} DESTINATION=TRANSIENT\n",
                    nick, port)
        },
        _ => todo!(),
    };

    match helper::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn stream(socket: &mut I2pStreamSocket, nick: &str) -> Result<(), I2pError> {

    let msg = format!("SESSION CREATE STYLE=STREAM ID={} DESTINATION=TRANSIENT\n", nick);

    match helper::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::I2pStreamSocket;

    #[test]
    fn test_cmd_session_create() {
        let mut socket = I2pStreamSocket::connected().unwrap();

        assert_eq!(
            stream(&mut socket, "nickname1"),
            Ok(())
        );
    }

    // try to create session and then another with the same nickname
    //
    // ignore for now as this takes several tens of seconds
    #[test]
    fn test_cmd_session_create_duplicate() {
        let mut socket = I2pStreamSocket::connected().unwrap();

        assert_eq!(
            stream(&mut socket, "nickname2"),
            Ok(())
        );

        assert_eq!(
            stream(&mut socket, "nickname2"),
            Err(I2pError::RouterError),
        );
    }

    // ignore for now as this takes several tens of seconds
    #[test]
    fn test_cmd_session_create_session_two_sockets_same_nick() {
        let mut socket1 = I2pStreamSocket::connected().unwrap();
        let mut socket2 = I2pStreamSocket::connected().unwrap();

        assert_eq!(
            stream(&mut socket1, "nickname3"),
            Ok(())
        );

        assert_eq!(
            stream(&mut socket2, "nickname3"),
            Err(I2pError::Duplicate),
        );
    }

    // try to create multiple datagram sessions for one session,
    // only the first one should succeed
    #[test]
    fn test_cmd_session_dgram_two_connections() {
        let mut socket = I2pStreamSocket::connected().unwrap();

        assert_eq!(
            datagram(&mut socket, &SessionType::AnonymousDatagram, "nickname4", 8888),
            Ok(()),
        );

        assert_eq!(
            datagram(&mut socket, &SessionType::AnonymousDatagram, "nickname4", 8888),
            Err(I2pError::RouterError),
        );

        assert_eq!(
            datagram(&mut socket, &SessionType::RepliableDatagram, "nickname4", 8888),
            Err(I2pError::RouterError),
        );

        assert_eq!(
            datagram(&mut socket, &SessionType::RepliableDatagram, "nickname4", 9999),
            Err(I2pError::RouterError),
        );
    }


    #[test]
    fn test_cmd_session_dgram_two_sockets() {
        let mut socket1 = I2pStreamSocket::connected().unwrap();
        let mut socket2 = I2pStreamSocket::connected().unwrap();

        assert_eq!(
            datagram(&mut socket1, &SessionType::AnonymousDatagram, "nickname5", 8888),
            Ok(()),
        );

        // same port should fail even if there are two sockets
        assert_eq!(
            datagram(&mut socket2, &SessionType::AnonymousDatagram, "nickname5", 8888),
            Err(I2pError::Duplicate),
        );

        // same nick but different port should be okay
        assert_eq!(
            datagram(&mut socket2, &SessionType::AnonymousDatagram, "nickname5", 9999),
            Ok(()),
        );
    }
}
