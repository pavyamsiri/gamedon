use crate::lexer::{Error, Keyword, State, Token};
use gamedon_lexer::{Lexer, WithSpan, WrapWithSpan};
use owo_colors::{OwoColorize, Stream, Style};

macro_rules! early_return {
    ($result:expr) => {
        match $result {
            Ok(token) => token,
            Err(err) => {
                return err;
            }
        }
    };
}

macro_rules! early_return_as_err {
    ($result:expr) => {
        match $result {
            Ok(token) => token,
            Err(err) => {
                return Err(err);
            }
        }
    };
}

macro_rules! impl_eat_if_num {
    ($name:ident, $num_type:ident) => {
        fn $name(&mut self) -> Result<Option<WithSpan<$num_type>>, Command> {
            let operand = early_return_as_err!(self.peek_token());
            let span = operand.get_span().clone();
            let lexeme = &self
                .lexer
                .get_lexeme(&span)
                .expect("span orginates from lexer so it is always in range.");

            let number = match operand.get_kind() {
                Token::Integer => lexeme
                    .parse::<$num_type>()
                    .unwrap_or_else(|_| panic!("should always parse {}.", stringify!($num_type))),
                Token::Hexadecimal => {
                    let lexeme = lexeme.trim_start_matches("0x").trim();
                    $num_type::from_str_radix(lexeme, 16)
                        .unwrap_or_else(|_| panic!("should always parse {lexeme}"))
                }
                _ => {
                    return Ok(None);
                }
            };

            let _ = self.next_token()?;

            Ok(Some(number.wrap(span)))
        }
    };
}

#[derive(Debug)]
pub enum Command {
    Step,
    Resume,
    Stack,
    Status,
    Doctor,
    Reask,
    Retry(String),
    RunInto(u16),
    RunIntoWhen { address: u16, opcode: u16 },
    Read(u16),
    BreakWrite { address: u16, value: Option<u8> },
    BreakRead { address: u16 },
    Exit,
    LastOp,
}

pub struct Parser<'a> {
    lexer: Lexer<'a, State>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::<State>::new(source),
        }
    }

    pub fn parse(mut self) -> Command {
        let command = early_return!(self.next_token());

        let lexeme = self
            .lexer
            .get_lexeme(command.get_span())
            .expect("span orginates from lexer so it is always in range.");

        match command.get_kind() {
            Token::Ident | Token::String | Token::Integer | Token::Hexadecimal => {
                Command::Retry(format!(
                    "`{}` is not a valid command.",
                    lexeme.if_supports_color(Stream::Stdout, |text| text
                        .style(Style::new().yellow()))
                ))
            }
            Token::Keyword(Keyword::Exit) => Command::Exit,
            Token::Keyword(Keyword::RunTo) => self.parse_runto(),
            Token::Keyword(Keyword::Status) => Command::Status,
            Token::Keyword(Keyword::Doctor) => Command::Doctor,
            Token::Keyword(Keyword::Read) => self.parse_read(),
            Token::Keyword(Keyword::Step) => Command::Step,
            Token::Keyword(Keyword::Stack) => Command::Stack,
            Token::Keyword(Keyword::Resume) => Command::Resume,
            Token::Keyword(Keyword::LastOp) => Command::LastOp,
            Token::Keyword(Keyword::Break) => self.parse_break(),
            Token::Eof => Command::Reask,
        }
    }

    fn parse_runto(&mut self) -> Command {
        let address = early_return!(self.expect_u16());
        let opcode = early_return!(self.eat_if_u16()).map(WithSpan::take_kind);
        if let Some(opcode) = opcode {
            Command::RunIntoWhen {
                address: address.take_kind(),
                opcode,
            }
        } else {
            Command::RunInto(address.take_kind())
        }
    }

    fn parse_read(&mut self) -> Command {
        let address = early_return!(self.expect_u16());
        Command::Read(address.take_kind())
    }

    fn parse_break(&mut self) -> Command {
        let on_write = early_return!(self.parse_break_mode());
        let address = early_return!(self.expect_u16()).take_kind();
        if on_write {
            let value = early_return!(self.eat_if_u8()).map(WithSpan::take_kind);
            Command::BreakWrite { address, value }
        } else {
            Command::BreakRead { address }
        }
    }

    fn parse_break_mode(&mut self) -> Result<bool, Command> {
        match early_return_as_err!(self.expect_ident()).take_kind() {
            "r" => Ok(false),
            "w" => Ok(true),
            mode => Err(Command::Retry(format!(
                "`{}` is not a valid break mode.",
                mode.if_supports_color(Stream::Stdout, |text| text.style(Style::new().yellow()))
            ))),
        }
    }

    fn handle_token(
        &mut self,
        token_or_error: Result<WithSpan<Token>, WithSpan<Error>>,
    ) -> Result<WithSpan<Token>, Command> {
        let command_token = match token_or_error {
            Ok(token) => token,
            Err(err) => {
                let lexeme = &self
                    .lexer
                    .get_lexeme(err.get_span())
                    .expect("span orginates from lexer so it is always in range.");
                match err.get_kind() {
                    Error::UnknownChar { c } => {
                        return Err(Command::Retry(format!(
                            "{} is not valid character.",
                            c.if_supports_color(Stream::Stdout, |text| text
                                .style(Style::new().yellow()))
                        )));
                    }
                    Error::UnterminatedString { delimiter } => {
                        return Err(Command::Retry(format!(
                            "{} is missing a closing {}.",
                            lexeme.if_supports_color(Stream::Stdout, |text| text
                                .style(Style::new().yellow())),
                            delimiter.if_supports_color(Stream::Stdout, |text| text
                                .style(Style::new().red())),
                        )));
                    }
                    Error::UnknownNumberPrefix { prefix } => {
                        return Err(Command::Retry(format!(
                            "{} uses an unknown number prefix {}.",
                            lexeme.if_supports_color(Stream::Stdout, |text| text
                                .style(Style::new().yellow())),
                            prefix.if_supports_color(Stream::Stdout, |text| text
                                .style(Style::new().red())),
                        )));
                    }
                    Error::IncompleteHexadecimal => {
                        return Err(Command::Retry(format!(
                            "{} is missing a valid hexadecimal digit.",
                            lexeme.if_supports_color(Stream::Stdout, |text| text
                                .style(Style::new().yellow()))
                        )));
                    }
                }
            }
        };
        Ok(command_token)
    }

    fn next_token(&mut self) -> Result<WithSpan<Token>, Command> {
        let token_or_error = self.lexer.next_token();
        self.handle_token(token_or_error)
    }

    fn peek_token(&mut self) -> Result<WithSpan<Token>, Command> {
        let token_or_error = self.lexer.peek_token();
        self.handle_token(token_or_error)
    }

    impl_eat_if_num!(eat_if_u8, u8);
    impl_eat_if_num!(eat_if_u16, u16);

    fn expect_u16(&mut self) -> Result<WithSpan<u16>, Command> {
        let operand = early_return_as_err!(self.next_token());
        let lexeme = &self
            .lexer
            .get_lexeme(operand.get_span())
            .expect("span orginates from lexer so it is always in range.");

        let number = match operand.get_kind() {
            Token::Integer => lexeme.parse::<u16>().expect("should always parse u16."),
            Token::Hexadecimal => {
                let lexeme = lexeme.trim_start_matches("0x").trim();
                u16::from_str_radix(lexeme, 16)
                    .unwrap_or_else(|_| panic!("should always parse {lexeme}"))
            }
            _ => {
                return Err(Command::Retry(format!(
                    "Expected a number but got {}",
                    lexeme.if_supports_color(Stream::Stdout, |text| text
                        .style(Style::new().yellow()))
                )));
            }
        };

        Ok(number.wrap(operand.get_span().clone()))
    }

    fn expect_ident(&mut self) -> Result<WithSpan<&str>, Command> {
        let operand = early_return_as_err!(self.next_token());
        let span = operand.get_span().clone();
        let lexeme = &self
            .lexer
            .get_lexeme(&span)
            .expect("span orginates from lexer so it is always in range.");

        let ident = match operand.take_kind() {
            Token::Ident => *lexeme,
            _ => {
                return Err(Command::Retry(format!(
                    "Expected an ident but got {}",
                    lexeme.if_supports_color(Stream::Stdout, |text| text
                        .style(Style::new().yellow()))
                )));
            }
        };

        Ok(ident.wrap(span))
    }
}
