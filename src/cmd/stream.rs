use crate::error::I2pError;
use crate::socket::I2pSocket;
use crate::parser::{Command, Subcommand, parse};

/// Parse and validate router's SAMv3-compatible response
///
/// If the message is valid, return TODO
///
/// # Arguments
/// `response` - Router's response in text format
///
fn parse_response(response: &str) -> Result<(), I2pError> {

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
                    Ok(())
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

///
/// TODO
///
/// # Arguments
///
/// `socket` - I2pSocket object created by the caller.
/// `msg` - SAMv3 message that is sent to the router
///
fn send_internal(socket: &mut I2pSocket, msg: &str) -> Result<(), I2pError> {

    match socket.write(msg.as_bytes()) {
        Ok(_)  => {},
        Err(e) => {
            eprintln!("Failed to send DEST command to the router: {:#?}", e);
            return Err(I2pError::TcpStreamError);
        }
    }

    let mut data = String::new();
    match socket.read_line(&mut data) {
        Ok(_) => { },
        Err(e) => {
            eprintln!("Failed to read response from router: {:#?}", e);
            return Err(e);
        }
    }

    parse_response(&data)
}

/// TODO
///
/// # Arguments
///
/// `socket` - I2pSocket object created by the caller
/// `nick` - Nickname of the client, generated during I2pSession creation
///
pub fn connect(socket: &mut I2pSocket, nick: &str, host: &str) -> Result<(), I2pError> {
    let msg = format!("STREAM CONNECT ID={} DESTINATION={} SILENT=false\n", host, nick);
    send_internal(socket, &msg)
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
    send_internal(socket, &msg)
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
