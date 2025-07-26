use super::{Error, State, Token, number::ZeroState};
use gamedon_lexer::{
    GeneralState, GeneralToken, LexerPutBack, LexerStateTransition, SourceChar, Span, WithSpan,
    WrapWithSpan, state::IdentState,
};

/// The normal lexer state.
pub(crate) struct NormalState {
    /// The byte offset of the state.
    pub(crate) start: usize,
}

impl NormalState {
    /// Execute the state given the source text and a source character.
    pub(crate) fn execute(
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
                new_state: Some(State::Normal(NormalState {
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

    /// Return the next lexer state assuming it is also the normal state.
    const fn next(next_char: &SourceChar) -> State {
        State::Normal(Self {
            start: next_char.next_offset(),
        })
    }

    /// Return the span from the start of the state with the given `length`.
    const fn span(&self, length: usize) -> Span {
        Span {
            start: self.start,
            length,
        }
    }
}
