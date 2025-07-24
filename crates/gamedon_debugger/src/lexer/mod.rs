use gamedon_lexer::{
    GeneralError, GeneralKeyword, GeneralState, GeneralToken, LexerPutBack, LexerState,
    LexerStateTransition, SourceChar, Span, TokenWithEnd, WithSpan, WrapWithSpan,
    state::{IdentState, StringState},
};
use number::{FirstHexDigitState, HexDigitState, ZeroState};
use thiserror::Error;
mod initial;
use initial::InitialState;
mod number;

#[derive(Debug, Clone)]
pub enum Keyword {
    Doctor,
    RunTo,
    Status,
    Exit,
    Step,
    Stack,
    Resume,
    Read,
    Break,
    LastOp,
}

impl GeneralKeyword for Keyword {
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

#[derive(Debug, Clone)]
pub enum Token {
    Integer,
    Hexadecimal,
    Ident,
    String,
    Keyword(Keyword),
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

#[derive(Debug, PartialEq, Eq, Error, Clone)]
pub enum Error {
    #[error("Encountered unknown character {c}")]
    UnknownChar { c: char },
    #[error("String is unterminated: expected {delimiter} to close the string.")]
    UnterminatedString { delimiter: char },
    #[error("Encountered unknown number prefix {prefix}")]
    UnknownNumberPrefix { prefix: char },
    #[error("The hexadecimal number is incomplete.")]
    IncompleteHexadecimal,
}

impl GeneralError for Error {
    fn unterminated_string(span: Span, delimiter: char) -> WithSpan<Self> {
        Self::UnterminatedString { delimiter }.wrap(span)
    }
}

pub enum State {
    Initial(InitialState),
    Ident(IdentState),
    String(StringState),
    Zero(ZeroState),
    FirstHexDigit(FirstHexDigitState),
    HexDigit(HexDigitState),
    Finished,
}

impl GeneralState for State {
    fn initial(start: usize) -> Self {
        Self::Initial(InitialState { start })
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
    type Keyword = Keyword;

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
            State::Initial(state) => state.execute(text, next_char),
            State::Ident(state) => state.execute(text, next_char),
            State::String(state) => state.execute(text, next_char),
            State::Zero(state) => state.execute(text, next_char),
            State::FirstHexDigit(state) => state.execute(text, next_char),
            State::HexDigit(state) => state.execute(text, next_char),
        }
    }
}
