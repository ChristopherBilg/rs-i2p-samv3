use crate::error::I2pError;
use crate::socket::I2pStreamSocket;
use crate::parser::{Command, Subcommand, parse};
use crate::cmd::helper;

/// Parse and validate router's SAMv3-compatible response
///
/// If the message is valid, extract and return the keypair
///
/// # Arguments
/// `response` - Router's response in text format
///
fn parser(response: &str) -> Result<Vec<(String, String)>, I2pError> {

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
            eprintln!("Router's response did not contain PUB!");
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

    Ok(vec![(pubkey, privkey)])
}

fn generate_internal(socket: &mut I2pStreamSocket, msg: &str) -> Result<(String, String), I2pError> {
    match helper::exchange_msg(socket, &msg, &parser) {
        Ok(v)  => Ok(v[0].clone()),
        Err(e) => Err(e),
    }
}

/// Handshake with the router to establish initial connection
///
/// # Arguments
///
/// `socket` - I2pStreamSocket object created by the caller
///
pub fn generate(socket: &mut I2pStreamSocket) -> Result<(String, String), I2pError> {
    generate_internal(socket, "DEST GENERATE SIGNATURE_TYPE=7\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::I2pStreamSocket;

    #[test]
    fn test_cmd_dest_generate() {
        let mut socket = I2pStreamSocket::connected().unwrap();

        match generate(&mut socket) {
            Ok(_)  => assert!(true),
            Err(e) => {
                eprintln!("test_cmd_dest_generate: {:#?}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_cmd_dest_generate_invalid() {
        let mut socket = I2pStreamSocket::connected().unwrap();

        match generate_internal(&mut socket,  "DEST GENERATE SIGNATURE_TYPE=13371338\n") {
            Ok(_)  => assert!(false),
            Err(_) => assert!(true),
        }
    }
}
