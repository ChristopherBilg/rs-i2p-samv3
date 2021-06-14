#[derive(Debug)]
pub enum I2pError {
    Unknown,
    TcpConnectionError,
    TcpStreamError,
    NotSupported,
    InvalidValue,
    RouterError,
    ParseError,
}
