use gamedon_lexer::{
    GeneralError, GeneralKeyword, GeneralState, GeneralToken, LexerPutBack, LexerState,
    LexerStateTransition, SourceChar, Span, TokenWithEnd, WithSpan, WrapWithSpan,
    state::{IdentState, StringState},
};
use normal::NormalState;
use number::{FirstHexDigitState, HexDigitState, ZeroState};
use thiserror::Error;

/// The normal lexer state.
mod normal;
/// States relating to lexing numbers.
mod number;

/// The keywords in the debugger command language.
#[derive(Debug, Clone)]
pub(crate) enum Keyword {
    /// The `doctor` keyword.
    Doctor,
    /// The `runto` keyword.
    RunTo,
    /// The `status` keyword.
    Status,
    /// The `exit` keyword.
    Exit,
    /// The `step` keyword.
    Step,
    /// The `stack` keyword.
    Stack,
    /// The `resume` keyword.
    Resume,
    /// The `read` keyword.
    Read,
    /// The `break` keyword.
    Break,
    /// The `lastop` keyword.
    LastOp,
}

impl GeneralKeyword for Keyword {
    /// Parse a string into a keyword.
    fn parse(lexeme: &str) -> Option<Self> {
        let kw = match lexeme {
            "runto" => Self::RunTo,
            "doctor" => Self::Doctor,
            "status" => Self::Status,
            "stack" => Self::Stack,
            "read" => Self::Read,
            "step" => Self::Step,
            "resume" => Self::Resume,
            "break" => Self::Break,
            "exit" => Self::Exit,
            "lastop" => Self::LastOp,
            _ => return None,
        };
        Some(kw)
    }
}

/// Types of tokens in the debugger command language.
#[derive(Debug, Clone)]
pub(crate) enum Token {
    /// An integer in base 10.
    Integer,
    /// An integer in base 16 including the leading `0x`.
    Hexadecimal,
    /// An identifier.
    Ident,
    /// A string.
    String,
    /// A keyword.
    Keyword(Keyword),
    /// End of file.
    Eof,
}

impl TokenWithEnd for Token {
    fn is_eof(&self) -> bool {
        matches!(self, Self::Eof)
    }
}

impl GeneralToken<Keyword> for Token {
    fn string(span: Span) -> WithSpan<Self> {
        Self::String.wrap(span)
    }

    fn ident(span: Span) -> WithSpan<Self> {
        Self::Ident.wrap(span)
    }

    fn keyword(span: Span, key: Keyword) -> WithSpan<Self> {
        Self::Keyword(key).wrap(span)
    }

    fn eof(offset: usize) -> WithSpan<Self> {
        Self::Eof.wrap(Span {
            start: offset,
            length: 0,
        })
    }
}

/// Errors that occur when lexing.
#[derive(Debug, PartialEq, Eq, Error, Clone)]
pub(crate) enum Error {
    /// The lexer has encountered an unknown character.
    #[error("Encountered unknown character {c}")]
    UnknownChar { c: char },
    /// The lexer has encountered an unterminated string.
    #[error("String is unterminated: expected {delimiter} to close the string.")]
    UnterminatedString { delimiter: char },
    /// The lexer has encountered an unknown leading prefix for numbers.
    #[error("Encountered unknown number prefix {prefix}")]
    UnknownNumberPrefix { prefix: char },
    /// The lexer has encountered an incomplete hexadecimal.
    #[error("The hexadecimal number is incomplete.")]
    IncompleteHexadecimal,
}

impl GeneralError for Error {
    fn unterminated_string(span: Span, delimiter: char) -> WithSpan<Self> {
        Self::UnterminatedString { delimiter }.wrap(span)
    }
}

/// The lexer state.
pub(crate) enum State {
    /// The normal state.
    Normal(NormalState),
    /// The identifer state.
    Ident(IdentState),
    /// The string state.
    String(StringState),
    /// The zero state entered when seeing `0` in the `Normal` state.
    Zero(ZeroState),
    /// The state entered when seeing `x` in the `Zero` state.
    FirstHexDigit(FirstHexDigitState),
    /// The state entered when seeing any hex digit in the `FirstHexDigit` state.
    HexDigit(HexDigitState),
    /// The lexer has finished.
    Finished,
}

impl GeneralState for State {
    fn initial(start: usize) -> Self {
        Self::Normal(NormalState { start })
    }

    fn string(start: usize, closing: char, should_escape: bool) -> Self {
        Self::String(StringState {
            start,
            closing,
            escaped: should_escape,
        })
    }

    fn finished() -> Self {
        Self::Finished
    }
}

impl LexerState for State {
    type Token = WithSpan<Token>;
    type Error = WithSpan<Error>;

    fn initial() -> Self {
        <Self as GeneralState>::initial(0)
    }

    fn execute(
        &self,
        text: &str,
        next_char: Option<SourceChar>,
    ) -> LexerStateTransition<Self, Self::Token, Self::Error>
    where
        Self: Sized,
    {
        match self {
            State::Finished => LexerStateTransition {
                new_state: None,
                token_or_error: Some(Ok(Token::Eof.wrap(Span {
                    start: text.len(),
                    length: 0,
                }))),
                put_back: LexerPutBack::None,
            },
            State::Normal(state) => state.execute(text, next_char),
            State::Ident(state) => state.execute(text, next_char),
            State::String(state) => state.execute(text, next_char),
            State::Zero(state) => state.execute(text, next_char),
            State::FirstHexDigit(state) => state.execute(text, next_char),
            State::HexDigit(state) => state.execute(text, next_char),
        }
    }
}
