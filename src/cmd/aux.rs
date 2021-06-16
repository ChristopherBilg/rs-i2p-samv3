use crate::error::I2pError;
use crate::socket::I2pSocket;

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
/// `socket` - I2pSocket object created by the caller.
/// `msg` - SAMv3 message that is sent to the router
/// `parser` - parser function which validates the received response
///
pub fn exchange_msg(
    socket: &mut I2pSocket,
    msg:    &str,
    parser: &dyn Fn(&str) -> Result<Vec<(String, String)>, I2pError>)
    -> Result<Vec<(String, String)>, I2pError> {

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

    parser(&data)
}
