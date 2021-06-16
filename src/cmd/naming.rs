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
pub fn lookup(socket: &mut I2pSocket, addr: &str) -> Result<(String, String), I2pError> {
    lookup_internal(socket, &format!("NAMING LOOKUP NAME={}\n", addr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::{I2pSocket, SocketType};
    use crate::cmd::hello::*;

    #[test]
    fn test_lookup() {
        let mut socket = match I2pSocket::new(SocketType::Tcp, "localhost", 7656) {
            Ok(v)  => v,
            Err(e) => {
                eprintln!("test_handshake: {:#?}", e);
                assert!(false);
                return;
            }
        };

        // enable connection to router
        handshake(&mut socket);

        assert_eq!(
           lookup(&mut socket, "zzz.i2p"),
           Ok((
               "ME".to_string(),
               "GKapJ8koUcBj~jmQzHsTYxDg2tpfWj0xjQTzd8BhfC9c3OS5fwPBNajgF-eOD6eCjFTqTlorlh7Hnd8kXj1qblUGXT-tDoR9~YV8dmXl51cJn9MVTRrEqRWSJVXbUUz9t5Po6Xa247Vr0sJn27R4KoKP8QVj1GuH6dB3b6wTPbOamC3dkO18vkQkfZWUdRMDXk0d8AdjB0E0864nOT~J9Fpnd2pQE5uoFT6P0DqtQR2jsFvf9ME61aqLvKPPWpkgdn4z6Zkm-NJOcDz2Nv8Si7hli94E9SghMYRsdjU-knObKvxiagn84FIwcOpepxuG~kFXdD5NfsH0v6Uri3usE3uSzpWS0EHmrlfoLr5uGGd9ZHwwCIcgfOATaPRMUEQxiK9q48PS0V3EXXO4-YLT0vIfk4xO~XqZpn8~PW1kFe2mQMHd7oO89yCk-3yizRG3UyFtI7-mO~eCI6-m1spYoigStgoupnC3G85gJkqEjMm49gUjbhfWKWI-6NwTj0ZnAAAA".to_string()
           ))
        );

        assert_eq!(
            lookup(&mut socket, "abcdefghijklmnopqrstuvwxyz234567abcdefghijklmnopqrst.b32.i2p"),
            Err(I2pError::InvalidValue)
        );
    }
}
