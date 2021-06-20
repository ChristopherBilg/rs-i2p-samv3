use crate::error::I2pError;
use crate::socket::I2pStreamSocket;
use crate::parser::{Command, Subcommand, parse};
use crate::session::SessionType;
use crate::cmd::aux;

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

    match aux::check_result(&parsed) {
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

pub fn create_dgram(
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

    match aux::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn create(socket: &mut I2pStreamSocket, stype: &SessionType, nick: &str) -> Result<(), I2pError> {

    let msg = format!("SESSION CREATE STYLE=STREAM ID={} DESTINATION=TRANSIENT\n", nick);

    match aux::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::{I2pStreamSocket, I2pControlSocket};

    #[test]
    fn test_create_session() {
        let mut socket = I2pStreamSocket::connected().unwrap();

        assert_eq!(
            create(&mut socket, &SessionType::VirtualStream, "rs-i2p-samv3-test"),
            Ok(())
        );
    }

    // try to create session and then another with the same nickname
    //
    // ignore for now as this takes several tens of seconds
    #[test]
    #[ignore]
    fn test_create_session_duplicate() {
        let mut socket = I2pStreamSocket::connected().unwrap();

        assert_eq!(
            create(&mut socket, &SessionType::VirtualStream, "rs-i2p-samv3-test"),
            Ok(())
        );

        assert_eq!(
            create(&mut socket, &SessionType::VirtualStream, "rs-i2p-samv3-test"),
            Err(I2pError::Unknown),
        );
    }

    // ignore for now as this takes several tens of seconds
    #[test]
    #[ignore]
    fn test_create_session_two_sockets_same_nick() {
        let mut socket1 = I2pStreamSocket::connected().unwrap();
        let mut socket2 = I2pStreamSocket::connected().unwrap();

        assert_eq!(
            create(&mut socket1, &SessionType::VirtualStream, "rs-i2p-samv3-test"),
            Ok(())
        );

        assert_eq!(
            create(&mut socket2, &SessionType::VirtualStream, "rs-i2p-samv3-test"),
            Err(I2pError::Duplicate),
        );
    }
}
