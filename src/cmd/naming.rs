use crate::error::I2pError;
use crate::socket::I2pSocket;
use crate::parser::{Command, Subcommand, parse};

/// Parse and validate router's response to NAMING message
///
/// # Arguments
/// `response` - Router's response in text format
///
fn parse_response(response: &str) -> Result<(String, String), I2pError> {

    let parsed = match parse(response, Command::Naming, Some(Subcommand::Reply)) {
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
                },
                "KEY_NOT_FOUND" => {
                    return Err(I2pError::DoesntExist);
                },
                "INVALID_KEY" | "INVALID" => { // TODO i2pd only?
                    return Err(I2pError::InvalidValue);
                }
                _ => {
                    todo!();
                }
            }
        },
        None => {
            eprintln!("Router response did not contain RESULT!");
            return Err(I2pError::InvalidValue);
        }
    }

    let value = match parsed.get_value("VALUE") {
        Some(v) => v.to_string(),
        None    => "".to_string(),
    };


    match parsed.get_value("NAME") {
        Some(v) => {
            return Ok((v.to_string(), value));
        },
        None => {
            eprintln!("Router's respone did not contain NAME!");
            return Err(I2pError::InvalidValue);
        }
    };
}

/// TODO
///
/// # Arguments
///
/// `socket` - I2pSocket object created by the caller.
/// `msg` - SAMv3 message that is sent to the router
///
fn lookup_internal(socket: &mut I2pSocket, msg: &str) -> Result<(String, String), I2pError> {

    match socket.write(msg.as_bytes()) {
        Ok(_)  => {},
        Err(e) => {
            eprintln!("Failed to send NAMING command to the router: {:#?}", e);
            return Err(I2pError::TcpStreamError);
        }
    }

    let mut data = String::with_capacity(128);
    match socket.read(&mut data) {
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
pub fn lookup(socket: &mut I2pSocket, addr: &str) -> Result<(String, String), I2pError> {
    lookup_internal(socket, &format!("NAMING LOOKUP NAME={}\n", addr))
}
