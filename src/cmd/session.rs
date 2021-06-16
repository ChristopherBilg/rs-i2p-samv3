use crate::error::I2pError;
use crate::socket::I2pSocket;
use crate::parser::{Command, Subcommand, parse};
use crate::session::SessionType;
use crate::cmd::aux;

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
fn parser(response: &str) -> Result<Vec<(String, String)>, I2pError> {

    match parse(response, Command::Session, Some(Subcommand::Status)) {
        Ok(_)  => Ok(Vec::new()),
        Err(e) => {
            eprintln!("Failed to parse response: {:#?}", e);
            return Err(I2pError::InvalidValue);
        }
    }
}

pub fn create(socket: &mut I2pSocket, stype: &SessionType, nick: &str) -> Result<(), I2pError> {

    let msg = match stype {
        SessionType::VirtualStream => {
            format!("SESSION CREATE STYLE=STREAM ID={} DESTINATION=TRANSIENT\n", nick)
        },
        _ => todo!(),
    };

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
    fn test_gen_keys() {
        assert!(true);
    }
}

