use crate::error::I2pError;
use crate::socket::I2pSocket;
use crate::parser::{Command, Subcommand, parse};

#[derive(Debug, PartialEq, Eq)]
pub struct KeyPair {
}

/// Parse and validate router's SAMv3-compatible response
///
/// If the message is valid, return the received keypair to caller
///
/// # Arguments
/// `response` - Router's response in text format
///
fn parse_response(response: &str) -> Result<(String, String), I2pError> {

    let parsed = match parse(response, Command::Dest, Some(Subcommand::Reply)) {
        Ok(v)  => v,
        Err(e) => {
            eprintln!("Failed to parse response: {:#?}", e);
            return Err(I2pError::InvalidValue);
        }
    };

    let pubkey = match parsed.get_value("PUB") {
        Some(v) => v.to_string(),
        None    => {
            eprintln!("Router's respone did not contain PUB!");
            return Err(I2pError::InvalidValue);
        }
    };

    let privkey = match parsed.get_value("PRIV") {
        Some(v) => v.to_string(),
        None    => {
            eprintln!("Router's respone did not contain PUB!");
            return Err(I2pError::InvalidValue);
        }
    };

    Ok((pubkey, privkey))
}

/// gen_keys_internal() sends the specified message to the router and reads a response
/// with a timeout.
///
/// gen_keys_internal() expects, as the spec requires, that the message the router sends
/// ends in a newline (\n)
///
/// When the message has been read, it's parsed and validated
///
/// # Arguments
///
/// `socket` - I2pSocket object created by the caller.
/// `msg` - SAMv3 message that is sent to the router
///
fn generate_internal(socket: &mut I2pSocket, msg: &str) -> Result<(String, String), I2pError> {

    match socket.write(msg.as_bytes()) {
        Ok(_)  => {},
        Err(e) => {
            eprintln!("Failed to send DEST command to the router: {:#?}", e);
            return Err(I2pError::TcpStreamError);
        }
    }

    let mut data = String::with_capacity(128);
    match socket.read_line(&mut data) {
        Ok(_) => { },
        Err(e) => {
            eprintln!("Failed to read response from router: {:#?}", e);
            return Err(e);
        }
    }

    parse_response(&data)
}

/// Handshake with the router to establish initial connection
///
/// # Arguments
///
/// `socket` - I2pSocket object created by the caller
///
pub fn generate(socket: &mut I2pSocket) -> Result<(String, String), I2pError> {
    generate_internal(socket, &format!("DEST GENERATE SIGNATURE_TYPE=7\n"))
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
