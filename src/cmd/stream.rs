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

/// TODO
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
    use crate::socket::{I2pSocket, SocketType};

    #[test]
    fn test_generate() {
        assert!(true);
    }
}
