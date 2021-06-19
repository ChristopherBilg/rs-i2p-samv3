use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::{alphanumeric1, one_of, char},
    combinator::{opt, cut},
    error::{context, ErrorKind, ParseError, VerboseError},
    multi::many0,
    sequence::{preceded, terminated, tuple},
    AsChar, IResult, InputTakeAtPosition,
};
use crate::error::I2pError;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Hello,
    Ping,
    Session,
    Dest,
    Naming,
    Stream,
}

impl From<&str> for Command {
    fn from(i: &str) -> Self {
        match &i[..] {
            "HELLO"   => Command::Hello,
            "PING"    => Command::Ping,
            "SESSION" => Command::Session,
            "DEST"    => Command::Dest,
            "NAMING"  => Command::Naming,
            "STREAM"  => Command::Stream,
            _ => unimplemented!("Command {} not supported", i),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Subcommand {
    Reply,
    Create,
    Status,
    Lookup,
}

impl From<&str> for Subcommand {
    fn from(i: &str) -> Self {
        match i {
            "REPLY"  => Subcommand::Reply,
            "CREATE" => Subcommand::Create,
            "STATUS" => Subcommand::Status,
            "LOOKUP" => Subcommand::Lookup,
            _ => unimplemented!("Subcommand {} not supported", i),
        }
    }
}

// TODO add support for this enum
#[derive(Debug, PartialOrd, PartialEq)]
pub enum ValueType {
    Str(String),
    Double(f64),
}

pub type KeyValuePair<'a> = Vec<(&'a str, &'a str)>;

#[derive(Debug, Eq, PartialEq)]
pub struct Message<'a> {
    cmd:     Command,
    sub_cmd: Option<Subcommand>,
    values:  Option<KeyValuePair<'a>>,
}

#[derive(Debug, Eq, PartialEq)]
// pub struct Datagram<'a> {
pub struct DatagramHeader<'a> {
    pub dest:     &'a str,
    // values:   Option<KeyValuePair<'a>>,
    // datagram: Vec<u8>,
}

impl Message<'_> {
    pub fn get_value(&self, key: &str) -> Option<&str> {
        match &self.values {
            Some(values) => {
                for (k, v) in values {
                    if *k == key {
                        return Some(v);
                    }
                }
                None
            },
            None => None,
        }
    }
}

// TODO add unicode support
pub fn keyvalue_long<T, E: ParseError<T>>(i: T) -> IResult<T, T, E> where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    i.split_at_position1_complete(
        |item| {
            let char_item = item.as_char();
            !(char_item == '-') && !char_item.is_alphanum() && !(char_item == '.') &&
            !(char_item == ' ') && !(char_item == '=') && !(char_item == '~') &&
            !(char_item == '_')

        },
        ErrorKind::AlphaNumeric,
    )
}

pub fn keyvalue<T, E: ParseError<T>>(i: T) -> IResult<T, T, E> where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    i.split_at_position1_complete(
        |item| {
            let char_item = item.as_char();
            !(char_item == '.') && !char_item.is_alphanum() && !(char_item == '-') &&
            !(char_item == '=') && !(char_item == '~') && !(char_item == '_')
        },
        ErrorKind::AlphaNumeric,
    )
}

fn whitespace<'a>(i: &'a str) -> Res<&'a str, &'a str> {
    take_while(move |c|  " \t\r\n".contains(c))(i)
}

fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    escaped(keyvalue_long, '\\', one_of("\"n\\"))(i)
}

fn key<'a>(i: &'a str) -> Res<&'a str, &'a str> {
    escaped(alphanumeric1, '\\', one_of("\"n\\"))(i)
}

fn value<'a>(i: &'a str) -> Res<&'a str, &'a str> {
    escaped(keyvalue, '\\', one_of("\"n\\"))(i)
}

fn value_quoted<'a>(i: &'a str) -> Res<&'a str, &'a str> {
    context(
        "value_quoted",
        preceded(char('\"'), cut(terminated(parse_str, char('\"')))),
    )(i)
}

fn command(input: &str) -> Res<&str, Command> {
    context(
        "command",
        alt((
            tag("HELLO"),  tag("PING"),
            tag("DEST"),   tag("SESSION"),
            tag("NAMING"), tag("STREAM"),
        )),
    )(input)
    .map(|(next_input, res)| (next_input, res.into()))
}

fn sub_command(input: &str) -> Res<&str, Subcommand> {
    context(
        "sub_command",
        alt((
            tag("REPLY"),  tag("CREATE"),
            tag("STATUS"), tag("LOOKUP"),
        )),
    )(input)
    .map(|(next_input, res)| (next_input, res.into()))
}

fn values<'a>(input: &str) -> Res<&str, KeyValuePair> {
    context(
        "key/value pairs",
        tuple((
            whitespace,
            key,
            tag("="),
            alt((value_quoted, value)),
            many0(tuple((
                whitespace,
                key,
                tag("="),
                alt((value_quoted, value)),
            ))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let mut qps = Vec::new();
        qps.push((res.1, res.3));
        for qp in res.4 {
            qps.push((qp.1, qp.3));
        }
        (next_input, qps)
    })
}

fn parse_internal(input: &str) -> Res<&str, Message> {
    context(
        "uri",
        tuple((
            command,
            opt(whitespace),
            opt(sub_command),
            opt(whitespace),
            opt(values),
        )),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            Message {
                cmd:     res.0,
                sub_cmd: res.2,
                values:  res.4,
            },
        )
    })
}

fn parse_header_internal(input: &str) -> Res<&str, DatagramHeader> {
    context(
        "header",
        tuple((
            value,
            // opt(whitespace),
            // opt(values),
            tag("\n")
        )),
    )(input)
    .map(|(next_input, res)| {
        (
            next_input,
            DatagramHeader {
                dest:    res.0,
            },
        )
    })
}

pub fn parse_header<'a>(data: &'a str) -> Result<(DatagramHeader, &'a str), I2pError> {
    match parse_header_internal(data) {
        Ok(v)  => return Ok((v.1, v.0)),
        Err(e) => {
            eprintln!("Failed to parse response: {:#?}", e);
            return Err(I2pError::ParseError);
        }
    };
}

pub fn parse(data: &str, cmd: Command, sub_cmd: Option<Subcommand>) -> Result<Message, I2pError> {

    match parse_internal(data) {
        Ok(v) => {
            if v.1.cmd == cmd && v.1.sub_cmd == sub_cmd {
                return Ok(v.1);
            }

            eprintln!("Did not receive expected reply from router: {:#?} {:#?}",
                      v.1.cmd, v.1.sub_cmd);
            return Err(I2pError::RouterError);
        },
        Err(e) => {
            eprintln!("Failed to parse response: {:#?}", e);
            return Err(I2pError::ParseError);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // test that commands and subcommands are parsed correctly
    #[test]
    fn test_cmd_subcmd() {

        assert_eq!(
            parse_internal("HELLO REPLY"),
            Ok(("",
                Message {
                    cmd:     Command::Hello,
                    sub_cmd: Some(Subcommand::Reply),
                    values:  None,
                }
            ))
        );

        assert_eq!(
            parse_internal("PING RESULT=OK"),
            Ok(("",
                Message {
                    cmd:     Command::Ping,
                    sub_cmd: None,
                    values:  Some(vec![
                        (
                            "RESULT",
                            "OK",
                        ),
                    ])
                }
            ))
        );
    }

    // test that both quoted and unquoted key/value pairs work
    #[test]
    fn test_quoted_unquoted() {
        let valid = Ok(("",
            Message {
                cmd: Command::Hello,
                sub_cmd: Some(
                    Subcommand::Reply,
                ),
                values: Some(vec![
                    (
                        "RESULT",
                        "OK",
                    ),
                    (
                        "MESSAGE",
                        "test hello 123",
                    ),
                ]),
            },
        ));

        assert_eq!(
            parse_internal("HELLO REPLY RESULT=\"OK\" MESSAGE=\"test hello 123\""),
            valid
        );

        assert_eq!(
            parse_internal("HELLO REPLY RESULT=OK MESSAGE=\"test hello 123\""),
            valid
        );
    }

    // test that decimal is parsed correctly
    #[test]
    fn test_decimal() {
        assert_eq!(
            parse_internal("HELLO VERSION=3.1 ADDR=google.com"),
            Ok(("",
                Message {
                    cmd:     Command::Hello,
                    sub_cmd: None,
                    values:  Some(vec![
                        (
                            "VERSION",
                            "3.1"
                        ),
                        (
                            "ADDR",
                            "google.com",
                        ),
                    ]),
                }
            ))
        );
    }

    #[test]
    fn test_get_value() {
        let parsed = parse_internal("HELLO VERSION=3.1 ADDR=google.com").unwrap().1;

        assert_eq!(parsed.get_value("VERSION"), Some("3.1"));
        assert_eq!(parsed.get_value("ADDR"),    Some("google.com"));
        assert_eq!(parsed.get_value("TEST"),    None);
    }

    #[test]
    fn test_parse() {

        // parse valid response
        assert_eq!(
            parse(
                "HELLO REPLY RESULT=OK VERSION=3.1",
                Command::Hello,
                Some(Subcommand::Reply)
            ),
            Ok(Message {
                cmd:     Command::Hello,
                sub_cmd: Some(Subcommand::Reply),
                values:  Some(vec![
                    (
                        "RESULT",
                        "OK",
                    ),
                    (
                        "VERSION",
                        "3.1",
                    ),
                ]),
            })
        );

        // parse invalid response (subcommand missing)
        assert_eq!(
            parse(
                "HELLO RESULT=OK VERSION=3.1",
                Command::Hello,
                Some(Subcommand::Reply)
            ),
            Err(I2pError::RouterError),
        );

        // parse invalid response (command and subcommand both missing)
        assert_eq!(
            parse(
                "3.1",
                Command::Hello,
                Some(Subcommand::Reply)
            ),
            Err(I2pError::ParseError),
        );
    }

        #[test]
        fn test_base64() {
            assert_eq!(
                parse(
                    "DEST REPLY PUB=B9pegw5QkKt2NcN~OxyUrtrZBprhmZHeZRRE33V3s-RWd7Rhg2lerMpByNwM9S5Z3I96SPfFz5thlvzP7JmnXPT85IcAJ2eYg~e9EgipIfg4os49lBzrukQ3e~wJyIpkooSuV3rEyXR9zk9JlNBxmDJnYyRxYZedOK9sT8~ScKReHRDNC~Gb6RyEnlR4RItWVWAuUCDegoLxUh~idZj704MgHE5zio1QTbxMsgBumvXxNmDf5Irc9YpTnfvuKiKc4uOEyzN96t~zkgVMCz4ttMchzJSeqWRxvoqmHTkjuSrhJ0vE3ON-UVn1LU3e-9jVKq-GDj3bUTEnSC6WfKcivcypmv-s7DkkezFdpu3HEBYtcjkJf~AFnpXCL1S1F6gUoUbsLlCl9PDGpXYMBhS0rrLfOj4dCiAZC9zbTo3OTp60dwg5be4fXTW~CeCEXwGzlTZlc~4P~rYfOQ8Fzs5vprTsD79gKYlCs9kPwCDJL2Tfv-ggVLKXber32f5OHUmfBQAEAAcAAA== PRIV=PUcsXtuhfPem9Fmf--eHA~nLHXzk9xn21cK5LOSW6H3dy9chBXveC2jeiGo6ERsX9WhGpMwHYu6waNJtHUm6GKKuDrK9nTTyxX8DSjCXKyseNzvZmgjuHVieQzLlTBqOAMkNvTzKUnawuIL3u~PtLTHoqPRllr13g3x-vG5K8Ll38UrHsq6prf7TNN12SkyUJPg0SvM-Fy5sd8hg-n~TAut5YA2dU0-bsvSTycBdBULzfsz1QgmLdVwzi~zFKCdjoPDiwsyVSAz2votd2U6oPXy-qiGaPZAun3tEfz7pFOVC94ZWW~166O~aLsNfVdEhAyW0z1RrTx-zhyynAY64FeGwLJyr010u7aXopXfhCvb2QzU4tSSHEAGXqzQbcbB0ztdHviZHwJpP32B-ZE3sfpEWLE9h3yPtiWG7qYyyax6sN44GfSIAoeq1M0O4hJ3whA~yI5dtzz6Orf49Y2h-53uvHvpVIisGfzXbesvP71PoN-XB2XL9IOdip3xF4HpRBQAEAAcAAA==",
                    Command::Dest,
                    Some(Subcommand::Reply)
                ),
                Ok(Message {
                    cmd:     Command::Dest,
                    sub_cmd: Some(Subcommand::Reply),
                    values:  Some(vec![
                        (
                            "PUB",
                            "B9pegw5QkKt2NcN~OxyUrtrZBprhmZHeZRRE33V3s-RWd7Rhg2lerMpByNwM9S5Z3I96SPfFz5thlvzP7JmnXPT85IcAJ2eYg~e9EgipIfg4os49lBzrukQ3e~wJyIpkooSuV3rEyXR9zk9JlNBxmDJnYyRxYZedOK9sT8~ScKReHRDNC~Gb6RyEnlR4RItWVWAuUCDegoLxUh~idZj704MgHE5zio1QTbxMsgBumvXxNmDf5Irc9YpTnfvuKiKc4uOEyzN96t~zkgVMCz4ttMchzJSeqWRxvoqmHTkjuSrhJ0vE3ON-UVn1LU3e-9jVKq-GDj3bUTEnSC6WfKcivcypmv-s7DkkezFdpu3HEBYtcjkJf~AFnpXCL1S1F6gUoUbsLlCl9PDGpXYMBhS0rrLfOj4dCiAZC9zbTo3OTp60dwg5be4fXTW~CeCEXwGzlTZlc~4P~rYfOQ8Fzs5vprTsD79gKYlCs9kPwCDJL2Tfv-ggVLKXber32f5OHUmfBQAEAAcAAA==",
                        ),
                        (
                            "PRIV",
                            "PUcsXtuhfPem9Fmf--eHA~nLHXzk9xn21cK5LOSW6H3dy9chBXveC2jeiGo6ERsX9WhGpMwHYu6waNJtHUm6GKKuDrK9nTTyxX8DSjCXKyseNzvZmgjuHVieQzLlTBqOAMkNvTzKUnawuIL3u~PtLTHoqPRllr13g3x-vG5K8Ll38UrHsq6prf7TNN12SkyUJPg0SvM-Fy5sd8hg-n~TAut5YA2dU0-bsvSTycBdBULzfsz1QgmLdVwzi~zFKCdjoPDiwsyVSAz2votd2U6oPXy-qiGaPZAun3tEfz7pFOVC94ZWW~166O~aLsNfVdEhAyW0z1RrTx-zhyynAY64FeGwLJyr010u7aXopXfhCvb2QzU4tSSHEAGXqzQbcbB0ztdHviZHwJpP32B-ZE3sfpEWLE9h3yPtiWG7qYyyax6sN44GfSIAoeq1M0O4hJ3whA~yI5dtzz6Orf49Y2h-53uvHvpVIisGfzXbesvP71PoN-XB2XL9IOdip3xF4HpRBQAEAAcAAA==",
                        ),
                    ]),
                })
            );
        }

        #[test]
        fn test_parse_header_valid1() {
            assert_eq!(
                parse_header(
                    "ABCDEFG FROMPORT=7777 TOPORT=8888\nHello, world!",
                ),
                Ok((
                    DatagramHeader {
                        dest:   "ABCDEFG",
                        // values: Some(vec![
                        //     (
                        //         "FROMPORT",
                        //         "7777",
                        //     ),
                        //     (
                        //         "TOPORT",
                        //         "8888",
                        //     ),
                        // ]),
                    },
                    "Hello, world!"
                ))
            );
        }

        #[test]
        fn test_parse_header_valid2() {
            assert_eq!(
                parse_header(
                    "ABCDEFG\nHello, world!",
                ),
                Ok((
                    DatagramHeader {
                        dest:   "ABCDEFG",
                    },
                    "Hello, world!"
                ))
            );
        }

}
