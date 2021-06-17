use crate::error::I2pError;
use crate::socket::I2pStreamSocket;
use crate::parser::Message;

/// exchange_msg() sends the specified message to the router and reads a response
/// with a timeout.
///
/// exchange_msg() expects, as the spec requires, that the message the router sends
/// ends in a newline (\n)
///
/// When the message has been read, it's passed on to the command-specific parser
/// which validates the message and returns all interesting fields as a vector
/// of key-value pairs
///
/// # Arguments
///
/// `socket` - I2pStreamSocket object created by the caller.
/// `msg` - SAMv3 message that is sent to the router
/// `parser` - parser function which validates the received response
///
pub fn exchange_msg(
    socket: &mut I2pStreamSocket,
    msg:    &str,
    parser: &dyn Fn(&str) -> Result<Vec<(String, String)>, I2pError>)
    -> Result<Vec<(String, String)>, I2pError> {

    match socket.write(msg.as_bytes()) {
        Ok(_)  => { },
        Err(e) => {
            eprintln!("Failed to send command command to the router: {:#?}", e);
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

    parser(&data)
}

fn get_message(response: &Message) -> String {
    match response.get_value("MESSAGE") {
        Some(v) => v.to_string(),
        None    => "No message from router".to_string(),
    }
}

pub fn check_result(response: &Message) -> Result<(), (I2pError, String)> {
    match response.get_value("RESULT") {
        Some(res) => {
            match &res[..] {
                "OK" => {
                    Ok(())
                },
                "DUPLICATED_ID" | "DUPLICATED_DEST" => {
                    Err((I2pError::Duplicate, get_message(&response)))
                },
                "INVALID_KEY" | "INVALID_ID" => {
                    Err((I2pError::InvalidValue, get_message(&response)))
                },
                "I2P_ERROR" => {
                    Err((I2pError::RouterError, get_message(&response)))
                },
                "KEY_NOT_FOUND" => {
                    Err((I2pError::DoesntExist, get_message(&response)))
                }
                _ => {
                    Err((I2pError::Unknown, get_message(&response)))
                }
            }
        },
        None => {
            Err((I2pError::DoesntExist, get_message(&response)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Message, Command, Subcommand, parse};

    #[test]
    fn test_get_message() {
        assert_eq!(
            get_message(&parse(
                "HELLO REPLY MESSAGE=\"HELLO WORLD\"",
                Command::Hello,
                Some(Subcommand::Reply),
            ).unwrap()),
            "HELLO WORLD",
        );

        assert_eq!(
            get_message(&parse(
                "HELLO REPLY VERSION=3.1",
                Command::Hello,
                Some(Subcommand::Reply),
            ).unwrap()),
            "No message from router",
        );
    }

    #[test]
    fn test_check_result() {
        assert_eq!(
            check_result(&parse(
                "HELLO REPLY",
                Command::Hello,
                Some(Subcommand::Reply),
            ).unwrap()),
            Err((I2pError::DoesntExist, "No message from router".to_string())),
        );

        assert_eq!(
            check_result(&parse(
                "HELLO RESULT=OK",
                Command::Hello,
                None,
            ).unwrap()),
            Ok(()),
        );

        assert_eq!(
            check_result(&parse(
                "HELLO RESULT=DUPLICATED_ID",
                Command::Hello,
                None,
            ).unwrap()),
            Err((I2pError::Duplicate, "No message from router".to_string())),
        );

        assert_eq!(
            check_result(&parse(
                "HELLO RESULT=DUPLICATED_DEST MESSAGE=\"DESTINATION ALREDY EXIST\"",
                Command::Hello,
                None,
            ).unwrap()),
            Err((
                I2pError::Duplicate,
                "DESTINATION ALREDY EXIST".to_string(),
            )),
        );

        assert_eq!(
            check_result(&parse(
                "HELLO RESULT=INVALID_KEY",
                Command::Hello,
                None,
            ).unwrap()),
            Err((I2pError::InvalidValue, "No message from router".to_string())),
        );

        assert_eq!(
            check_result(&parse(
                "HELLO RESULT=INVALID_ID MESSAGE=\"INVALID NICKNAME\"",
                Command::Hello,
                None,
            ).unwrap()),
            Err((
                I2pError::InvalidValue,
                "INVALID NICKNAME".to_string(),
            )),
        );

        assert_eq!(
            check_result(&parse(
                "HELLO RESULT=I2P_ERROR MESSAGE=\"ROUTER ERROR\"",
                Command::Hello,
                None,
            ).unwrap()),
            Err((
                I2pError::RouterError,
                "ROUTER ERROR".to_string(),
            )),
        );

        assert_eq!(
            check_result(&parse(
                "HELLO REPLY RESULT=INVALID_RESULT MESSAGE=\"NEW STATUS CODE\"",
                Command::Hello,
                Some(Subcommand::Reply),
            ).unwrap()),
            Err((
                I2pError::Unknown,
                "NEW STATUS CODE".to_string(),
            )),
        );
    }
}
