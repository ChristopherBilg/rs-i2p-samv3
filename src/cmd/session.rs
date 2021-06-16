use crate::error::I2pError;
use crate::socket::I2pSocket;
use crate::parser::{Command, Subcommand, parse};
use crate::session::SessionType;

#[derive(Debug, PartialEq, Eq)]
pub struct KeyPair {
}

/// Parse and validate router's SAMv3-compatible response
///
/// If the message is valid, return the parsed Message object to caller
///
/// # Arguments
/// `response` - Router's response in text format
///
fn parse_response(response: &str) -> Result<(), I2pError> {

    match parse(response, Command::Session, Some(Subcommand::Status)) {
        Ok(_)  => Ok(()),
        Err(e) => {
            eprintln!("Failed to parse response: {:#?}", e);
            return Err(I2pError::InvalidValue);
        }
    }
}

/// TODO
fn create_internal(socket: &mut I2pSocket, msg: &str) -> Result<(), I2pError> {
    match socket.write(msg.as_bytes()) {
        Ok(_)  => { },
        Err(e) => {
            eprintln!("Failed to send DEST command to the router: {:#?}", e);
            return Err(I2pError::TcpStreamError);
        }
    }

    let mut data = String::new();
    match socket.read_line(&mut data) {
        Ok(_)  => { },
        Err(e) => {
            eprintln!("Failed to read response from router: {:#?}", e);
            return Err(e);
        }
    }

    parse_response(&data)
}

/// TODO
pub fn create(socket: &mut I2pSocket, stype: &SessionType, nick: &str) -> Result<(), I2pError> {

    let msg = match stype {
        SessionType::VirtualStream => {
            format!("SESSION CREATE STYLE=STREAM ID={} DESTINATION=TRANSIENT\n", nick)
        },
        _ => todo!(),
    };

    create_internal(socket, &msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::{I2pSocket, SocketType};

    #[test]
    fn test_gen_keys() {
        assert!(true);
    }
}

