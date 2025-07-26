use crate::lexer::{Error, Keyword, State, Token};
use gamedon_lexer::{Lexer, WithSpan, WrapWithSpan};
use owo_colors::{OwoColorize, Stream, Style};

/// Unwrap a result and return the error directly.
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

/// Unwrap a result and return the error wrapped as an error variant of a result.
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

/// Implement the method `eat_if_{num}` where `num` is a numeric type that can be parsed from string.
/// The function is meant to peek the current lexed token and consume it if and only if the token can
/// be parsed into the given numeric type.
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

/// All debugger commands.
#[derive(Debug)]
pub enum Command {
    /// Step the CPU ahead one whole instruction.
    Step,
    /// Resume CPU execution until it hits a breakpoint.
    Resume,
    /// Print the stack pointer and the word pointed to by it.
    Stack,
    /// Print the status of the emulator.
    Status,
    /// Print the status of the emulator in a format compatible with `gameboy-doctor`.
    Doctor,
    /// Reask the user without displaying anything.
    Reask,
    /// Reask the user while displaying the inner string.
    Retry(String),
    /// Add a breakpoint that triggers when the CPU is about to decode the instruction at `address`.
    RunInto(u16),
    /// Add a breakpoint that triggers when the CPU is about to execute the instruction at `address`
    /// with opcode equal to `opcode`.
    RunIntoWhen {
        /// The breakpoint address.
        address: u16,
        /// The opcode of the instruction to break against.
        opcode: u16,
    },
    /// Print the byte at `address`.
    Read(u16),
    /// Add a breakpoint that triggers when a byte is being written to `address`.
    BreakWrite {
        /// The breakpoint address.
        address: u16,
        /// If set the value to check against when the byte is being written to `address`. If the byte
        /// is the same as `value` then the breakpoint is triggered.
        value: Option<u8>,
    },
    /// Add a breakpoint that triggers when the byte at `address` is being read from the bus.
    BreakRead {
        /// The breakpoint address.
        address: u16,
    },
    /// Exit the debug command prompt.
    Exit,
    /// Print the last instruction executed by the CPU.
    LastOp,
}

/// A parser for the debug command language.
pub struct Parser<'a> {
    /// The lexer.
    lexer: Lexer<'a, State>,
}

impl<'a> Parser<'a> {
    /// Initialise the parser given the `source` text.
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::<State>::new(source),
        }
    }

    /// Parse the source into a `Command`.
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

    /// Parse the `runto` command assuming the previous token as the `runto` keyword.
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

    /// Parse the `read` command assuming the previous token as the `read` keyword.
    fn parse_read(&mut self) -> Command {
        let address = early_return!(self.expect_u16());
        Command::Read(address.take_kind())
    }

    /// Parse the `break` command assuming the previous token as the `break` keyword.
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

    /// Parse the mode of the bus read/write breakpoint where `true` means on write and `false` means on read.
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

    /// Convert a lexer token or error into a token or a retry command.
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

    /// Return the next lexed token or a retry command in the case of failure.
    fn next_token(&mut self) -> Result<WithSpan<Token>, Command> {
        let token_or_error = self.lexer.next_token();
        self.handle_token(token_or_error)
    }

    /// Return the current lexed token or a retry command in the case of failure.
    fn peek_token(&mut self) -> Result<WithSpan<Token>, Command> {
        let token_or_error = self.lexer.peek_token();
        self.handle_token(token_or_error)
    }

    impl_eat_if_num!(eat_if_u8, u8);
    impl_eat_if_num!(eat_if_u16, u16);

    /// Check that the next token can be converted into a `u16` otherwise return a retry command.
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

    /// Check that the next token is an identifier otherwise return a retry command.
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
