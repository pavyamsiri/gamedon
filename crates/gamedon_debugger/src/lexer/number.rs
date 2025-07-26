use super::{Error, State, Token};
use gamedon_lexer::{
    GeneralState, LexerPutBack, LexerStateTransition, SourceChar, Span, WithSpan, WrapWithSpan,
};

/// State when we see `0` in initial state.
pub(crate) struct ZeroState {
    /// The byte offset of the state.
    pub(crate) start: usize,
}

impl ZeroState {
    /// Execute the state given the source text and a source character.
    pub(crate) fn execute(
        &self,
        text: &str,
        next_char: Option<SourceChar>,
    ) -> LexerStateTransition<State, WithSpan<Token>, WithSpan<Error>> {
        let _ = text;
        let Some(next_char) = next_char else {
            // EOF
            return LexerStateTransition {
                new_state: Some(State::Finished),
                token_or_error: Some(Ok(Token::Integer.wrap(Span {
                    start: self.start,
                    length: '0'.len_utf8(),
                }))),
                put_back: LexerPutBack::None,
            };
        };

        let c = next_char.value;
        let len_utf8 = c.len_utf8();

        match c {
            'x' | 'X' => LexerStateTransition {
                new_state: Some(State::FirstHexDigit(FirstHexDigitState {
                    start: self.start,
                })),
                token_or_error: None,
                put_back: LexerPutBack::None,
            },
            _ => LexerStateTransition {
                new_state: Some(<State as GeneralState>::initial(next_char.next_offset())),
                token_or_error: Some(Err(Error::UnknownNumberPrefix {
                    prefix: next_char.value,
                }
                .wrap(self.span(len_utf8)))),
                put_back: LexerPutBack::None,
            },
        }
    }

    /// Return the span from the start of the state with the given `length`.
    const fn span(&self, length: usize) -> Span {
        Span {
            start: self.start,
            length: '0'.len_utf8() + length,
        }
    }
}

/// State when we see `0x` or `0X` in initial state.
pub(crate) struct FirstHexDigitState {
    /// The byte offset of the state.
    pub(crate) start: usize,
}

impl FirstHexDigitState {
    /// Execute the state given the source text and a source character.
    pub(crate) fn execute(
        &self,
        text: &str,
        next_char: Option<SourceChar>,
    ) -> LexerStateTransition<State, WithSpan<Token>, WithSpan<Error>> {
        let Some(next_char) = next_char else {
            let err = Error::IncompleteHexadecimal.wrap(Span {
                start: self.start,
                length: text.len() - self.start,
            });
            return LexerStateTransition {
                new_state: Some(<State as GeneralState>::initial(text.len())),
                token_or_error: Some(Err(err)),
                put_back: LexerPutBack::None,
            };
        };

        let span = Span {
            start: self.start,
            length: next_char.next_offset() - self.start,
        };

        if next_char.value.is_ascii_hexdigit() {
            LexerStateTransition {
                new_state: Some(State::HexDigit(HexDigitState { start: self.start })),
                token_or_error: None,
                put_back: LexerPutBack::None,
            }
        } else {
            let err = Error::IncompleteHexadecimal.wrap(span);
            LexerStateTransition {
                new_state: Some(<State as GeneralState>::initial(next_char.offset)),
                token_or_error: Some(Err(err)),
                put_back: LexerPutBack::One([next_char]),
            }
        }
    }
}

/// State when we see `0x` or `0X` and a hexdigit.
pub(crate) struct HexDigitState {
    /// The byte offset of the state.
    pub(crate) start: usize,
}

impl HexDigitState {
    /// Execute the state given the source text and a source character.
    pub(crate) fn execute(
        &self,
        text: &str,
        next_char: Option<SourceChar>,
    ) -> LexerStateTransition<State, WithSpan<Token>, WithSpan<Error>> {
        let Some(next_char) = next_char else {
            let token = Token::Hexadecimal.wrap(Span {
                start: self.start,
                length: text.len() - self.start,
            });
            return LexerStateTransition {
                new_state: Some(<State as GeneralState>::initial(text.len())),
                token_or_error: Some(Ok(token)),
                put_back: LexerPutBack::None,
            };
        };

        let span = Span {
            start: self.start,
            length: next_char.next_offset() - self.start,
        };

        if next_char.value.is_ascii_hexdigit() || next_char.value == '_' {
            LexerStateTransition {
                new_state: None,
                token_or_error: None,
                put_back: LexerPutBack::None,
            }
        } else {
            let token = Token::Hexadecimal.wrap(span);
            LexerStateTransition {
                new_state: Some(<State as GeneralState>::initial(next_char.offset)),
                token_or_error: Some(Ok(token)),
                put_back: LexerPutBack::One([next_char]),
            }
        }
    }
}
