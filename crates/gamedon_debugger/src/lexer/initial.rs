use super::{Error, State, Token, number::ZeroState};
use gamedon_lexer::{
    GeneralState, GeneralToken, LexerPutBack, LexerStateTransition, SourceChar, Span, WithSpan,
    WrapWithSpan, state::IdentState,
};

pub struct InitialState {
    pub start: usize,
}

impl InitialState {
    pub fn execute(
        &self,
        text: &str,
        next_char: Option<SourceChar>,
    ) -> LexerStateTransition<State, WithSpan<Token>, WithSpan<Error>> {
        let Some(next_char) = next_char else {
            // EOF
            return LexerStateTransition {
                new_state: Some(State::Finished),
                token_or_error: Some(Ok(Token::eof(text.len()))),
                put_back: LexerPutBack::None,
            };
        };

        let c = next_char.value;
        let offset = next_char.offset;
        let len_utf8 = c.len_utf8();

        match c {
            '\t' | '\x0C' | '\r' | ' ' | '\n' => LexerStateTransition {
                new_state: Some(State::Initial(InitialState {
                    start: next_char.next_offset(),
                })),
                token_or_error: None,
                put_back: LexerPutBack::None,
            },
            'a'..='z' | 'A'..='Z' => LexerStateTransition {
                new_state: Some(State::Ident(IdentState { start: offset })),
                token_or_error: None,
                put_back: LexerPutBack::None,
            },
            '0' => LexerStateTransition {
                new_state: Some(State::Zero(ZeroState { start: offset })),
                token_or_error: None,
                put_back: LexerPutBack::None,
            },
            c @ ('\'' | '"') => LexerStateTransition {
                new_state: Some(State::string(offset, c, false)),
                token_or_error: None,
                put_back: LexerPutBack::None,
            },
            _ => LexerStateTransition {
                new_state: Some(Self::next(&next_char)),
                token_or_error: Some(Err(
                    Error::UnknownChar { c: next_char.value }.wrap(self.span(len_utf8))
                )),
                put_back: LexerPutBack::None,
            },
        }
    }

    const fn next(next_char: &SourceChar) -> State {
        State::Initial(Self {
            start: next_char.next_offset(),
        })
    }

    const fn span(&self, length: usize) -> Span {
        Span {
            start: self.start,
            length,
        }
    }
}
