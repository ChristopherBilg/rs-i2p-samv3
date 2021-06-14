use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::{alpha1, alphanumeric1, one_of, char},
    combinator::{opt, cut},
    error::{context, ErrorKind, ContextError, ParseError, VerboseError},
    multi::{count, many0, many1, many_m_n},
    number::complete::double,
    sequence::{preceded, separated_pair, terminated, tuple},
    AsChar, Err as NomErr, IResult, InputTakeAtPosition,
};
use crate::error::I2pError;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Hello,
    Ping,
}

impl From<&str> for Command {
    fn from(i: &str) -> Self {
        match i {
            "HELLO" => Command::Hello,
            "PING"  => Command::Ping,
            _ => unimplemented!("Command {} not supported", i),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Subcommand {
    Reply
}

impl From<&str> for Subcommand {
    fn from(i: &str) -> Self {
        match i {
            "REPLY" => Subcommand::Reply,
            _ => unimplemented!("Subcommand {} not supported", i),
        }
    }
}

// TODO add support for this enum
#[derive(Debug)]
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

impl Message<'_> {
    pub fn get_value(&self, key: &str) -> Option<&str> {
        None
    }
}

// TODO add unicode support
pub fn keyvalue<T, E: ParseError<T>>(i: T) -> IResult<T, T, E> where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    i.split_at_position1_complete(
        |item| {
            let char_item = item.as_char();
            !(char_item == '-') && !char_item.is_alphanum() && !(char_item == '.') && !(char_item == ' ')
        },
        ErrorKind::AlphaNumeric,
    )
}

fn whitespace<'a>(i: &'a str) -> Res<&'a str, &'a str> {
    take_while(move |c|  " \t\r\n".contains(c))(i)
}

fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    escaped(keyvalue, '\\', one_of("\"n\\"))(i)
}

fn key<'a>(i: &'a str) -> Res<&'a str, &'a str> {
    escaped(alphanumeric1, '\\', one_of("\"n\\"))(i)
}

fn value<'a>(i: &'a str) -> Res<&'a str, &'a str> {
    escaped(alphanumeric1, '\\', one_of("\"n\\"))(i)
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
        alt((tag("HELLO"), tag("PING"))),
    )(input)
    .map(|(next_input, res)| (next_input, res.into()))
}

fn sub_command(input: &str) -> Res<&str, Subcommand> {
    context(
        "sub_command",
        alt((tag("REPLY"), tag("PING"))), // TODO remove ping
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

pub fn parse(data: &str, cmd: Command, sub_cmd: Option<Subcommand>) -> Result<Message, I2pError> {
    Err(I2pError::InvalidValue)
}