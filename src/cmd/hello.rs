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

fn handshake_internal(socket: &mut I2pSocket, msg: &str) -> Result<(), I2pError> {
    match aux::exchange_msg(socket, &msg, &parser) {
        Ok(_)  => Ok(()),
        Err(e) => Err(e),
    }
}

///
/// # Arguments
///
/// `socket` - I2pSocket object created by the caller
///
pub fn handshake(socket: &mut I2pSocket) -> Result<(), I2pError> {
    handshake_internal(
        socket,
        &format!("HELLO VERSION MIN={} MAX={}\n", MIN_VERSION, MAX_VERSION)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::{I2pSocket, SocketType, new_uninit};

    #[test]
    fn test_handshake() {
        let mut socket = new_uninit(SocketType::Tcp, "localhost", 7656).unwrap();

        assert_eq!(
            handshake_internal(&mut socket, "HELLO VERSION MIN=3.1 MAX=3.1\n"),
            Ok(())
        );

    }

    #[test]
    fn test_handshake_no_version() {
        let mut socket = new_uninit(SocketType::Tcp, "localhost", 7656).unwrap();
        assert_eq!(
            handshake_internal(&mut socket, "HELLO VERSION\n"),
            Ok(())
        );
    }

    #[test]
    fn test_handshake_min() {
        let mut socket = new_uninit(SocketType::Tcp, "localhost", 7656).unwrap();
        assert_eq!(
            handshake_internal(&mut socket, "HELLO VERSION MIN=3.1\n"),
            Ok(())
        );
    }

    #[test]
    fn test_handshake_max() {
        let mut socket = new_uninit(SocketType::Tcp, "localhost", 7656).unwrap();
        assert_eq!(
            handshake_internal(&mut socket, "HELLO VERSION MAX=3.1\n"),
            Ok(())
        );
    }

    #[test]
    fn test_handshake_invalid_subcommand() {
        let mut socket = new_uninit(SocketType::Tcp, "localhost", 7656).unwrap();
        assert_eq!(
            handshake_internal(&mut socket, "HELLO TEST\n"),
            Err(I2pError::RouterError),
        );
    }

    #[test]
    fn test_handshake_version_too_high() {
        let mut socket = new_uninit(SocketType::Tcp, "localhost", 7656).unwrap();
        assert_eq!(
            handshake_internal(&mut socket, "HELLO MIN=3.4\n"),
            Err(I2pError::RouterError),
        );
    }

    #[test]
    fn test_handshake_versions_switched() {
        let mut socket = new_uninit(SocketType::Tcp, "localhost", 7656).unwrap();
        assert_eq!(
            handshake_internal(&mut socket, "HELLO MIN=3.3 MAX=3.1\n"),
            Err(I2pError::RouterError),
        );
    }
}
