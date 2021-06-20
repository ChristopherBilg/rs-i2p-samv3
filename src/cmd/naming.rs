use crate::error::I2pError;
use crate::socket::I2pStreamSocket;
use crate::parser::{Command, Subcommand, parse};
use crate::cmd::aux;

fn parser(response: &str) -> Result<Vec<(String, String)>, I2pError> {

    let parsed = match parse(response, Command::Naming, Some(Subcommand::Reply)) {
        Ok(v)  => v,
        Err(e) => {
            eprintln!("Failed to parse response: {:#?}", e);
            return Err(I2pError::InvalidValue);
        }
    };

    match aux::check_result(&parsed) {
        Ok(_)  => { },
        Err(e) => {
            eprintln!("Response did not contain RESULT=OK: {:#?}", e.0);
            eprintln!("Message: {}", e.1);
            return Err(e.0);
        }
    }

    let value = match parsed.get_value("VALUE") {
        Some(v) => v.to_string(),
        None    => "".to_string(),
    };

    match parsed.get_value("NAME") {
        Some(v) => {
            return Ok(vec![(v.to_string(), value)]);
        },
        None => {
            eprintln!("Router's respone did not contain NAME!");
            return Err(I2pError::InvalidValue);
        }
    };
}

/// Handshake with the router to establish initial connection
///
/// # Arguments
///
/// `socket` - I2pStreamSocket object created by the caller
///
pub fn lookup(socket: &mut I2pStreamSocket, addr: &str) -> Result<(String, String), I2pError> {
    let msg = format!("NAMING LOOKUP NAME={}\n", addr);

    match aux::exchange_msg(socket, &msg, &parser) {
        Ok(v)  => Ok(v[0].clone()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::{I2pStreamSocket, I2pControlSocket};
    use crate::cmd::hello::*;
    use crate::session::*;
    use crate::error::I2pError;

    #[test]
    fn test_lookup() {
        let mut socket = I2pStreamSocket::connected().unwrap();

        // zzz.i2p exists
        assert_eq!(
           lookup(&mut socket, "zzz.i2p").unwrap().0,
            "zzz.i2p".to_string(),
        );

        assert_eq!(
            lookup(&mut socket, "abcdefghijklmnopqrstuvwxyz234567abcdefghijklmnopqrst.b32.i2p"),
            Err(I2pError::DoesntExist)
        );
    }

    // calls from the same socket to destination ME should result in the same public key
    #[test]
    fn test_lookup_same_socket() {
        let mut session = I2pSession::new(SessionType::VirtualStream).unwrap();

        assert_eq!(
            lookup(&mut session.socket, "ME").unwrap().0,
            "ME",
        );

        assert_eq!(
            lookup(&mut session.socket, "ME"),
            lookup(&mut session.socket, "ME"),
        );
    }

    // two separate connections, even from the same machine, should get different destinations
    #[test]
    fn test_lookup_two_sockets() {
        let mut session1 = I2pSession::new(SessionType::VirtualStream).unwrap();
        let mut session2 = I2pSession::new(SessionType::VirtualStream).unwrap();

        assert_ne!(
            lookup(&mut session1.socket, "ME"),
            lookup(&mut session2.socket, "ME"),
        );
    }
}
