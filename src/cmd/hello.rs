use crate::error::I2pError;
use crate::socket::I2pSocket;
use crate::parser::{Command, Subcommand, parse};
use crate::cmd::aux;

static MIN_VERSION: &'static str = "3.1";
static MAX_VERSION: &'static str = "3.1";

/// Parse and validate router's SAMv3-compatible response
///
/// # Arguments
/// `response` - Router's response in text format
///
fn parser(response: &str) -> Result<Vec<(String, String)>, I2pError> {

    let parsed = match parse(response, Command::Hello, Some(Subcommand::Reply)) {
        Ok(v)  => v,
        Err(e) => {
            eprintln!("Failed to parse response: {:#?}", e);
            return Err(I2pError::InvalidValue);
        }
    };

    match parsed.get_value("RESULT") {
        Some(v) => {
            match &v[..] {
                "OK" => {
                    Ok(Vec::new())
                },
                "I2P_ERROR" => {
                    return Err(I2pError::RouterError);
                },
                "NOVERSION" => {
                    return Err(I2pError::InvalidValue);
                },
                _ => {
                    eprintln!("Unknown response from router: {}", v);
                    return Err(I2pError::Unknown);
                }
            }
        },
        None => {
            eprintln!("Router respones did not contain RESULT!");
            return Err(I2pError::InvalidValue);
        }
    }
}

///
/// # Arguments
///
/// `socket` - I2pSocket object created by the caller
///
pub fn handshake(socket: &mut I2pSocket) -> Result<(), I2pError> {
    let msg = format!("HELLO VERSION MIN={} MAX={}\n", MIN_VERSION, MAX_VERSION);

    match aux::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::{I2pSocket, SocketType};

    #[test]
    fn test_handshake() {
        let mut socket = match I2pSocket::new(SocketType::Tcp, "localhost", 7656) {
            Ok(v)  => v,
            Err(e) => {
                eprintln!("test_handshake: {:#?}", e);
                assert!(false);
                return;
            }
        };

        match handshake(&mut socket) {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("test_handshake: {:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_handshake_no_version() {
        let mut socket = match I2pSocket::new(SocketType::Tcp, "localhost", 7656) {
            Ok(v)  => v,
            Err(e) => {
                eprintln!("test_handshake: {:#?}", e);
                assert!(false);
                return;
            }
        };

        assert_eq!(
            handshake_internal(&mut socket, "HELLO VERSION"),
            Ok(())
        );
    }

    #[test]
    fn test_hello_invalid_message() {
        let mut socket = match I2pSocket::new(SocketType::Tcp, "localhost", 7656) {
            Ok(v)  => v,
            Err(e) => {
                eprintln!("test_handshake: {:#?}", e);
                assert!(false);
                return;
            }
        };

        assert_eq!(
            handshake_internal(&mut socket, "HELLO test"),
            Err(I2pError::InvalidValue)
        );

        assert_eq!(
            handshake_internal(&mut socket, "HELLO VERSION MIN=3.1 MAX=2.8"),
            Err(I2pError::InvalidValue)
        );
    }
}
