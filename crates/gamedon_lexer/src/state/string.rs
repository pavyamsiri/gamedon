use crate::{
    GeneralError, GeneralKeyword, GeneralState, GeneralToken, LexerPutBack, LexerStateTransition,
    SourceChar, Span, WithSpan,
};

#[derive(Debug)]
pub struct StringState {
    pub start: usize,
    pub closing: char,
    pub escaped: bool,
}

impl StringState {
    #[inline]
    pub fn execute<S: GeneralState, T: GeneralToken<K>, E: GeneralError, K: GeneralKeyword>(
        &self,
        text: &str,
        next_char: Option<SourceChar>,
    ) -> LexerStateTransition<S, WithSpan<T>, WithSpan<E>> {
        let Some(next_char) = next_char else {
            return LexerStateTransition {
                new_state: Some(GeneralState::initial(text.len())),
                token_or_error: Some(Err(E::unterminated_string(
                    Span {
                        start: self.start,
                        length: text.len() - self.start,
                    },
                    self.closing,
                ))),
                put_back: LexerPutBack::None,
            };
        };

        let span = Span {
            start: self.start,
            length: next_char.next_offset() - self.start,
        };

        // Found delimiter
        if next_char.value == self.closing {
            // Escaped
            if self.escaped {
                LexerStateTransition {
                    new_state: Some(S::string(self.start, self.closing, false)),
                    token_or_error: None,
                    put_back: LexerPutBack::None,
                }
            } else {
                LexerStateTransition {
                    new_state: Some(S::initial(next_char.next_offset())),
                    token_or_error: Some(Ok(T::string(span))),
                    put_back: LexerPutBack::None,
                }
            }
        }
        // Hit newline without terminating
        else if next_char.value == '\n' {
            LexerStateTransition {
                new_state: Some(S::initial(next_char.next_offset())),
                token_or_error: Some(Err(E::unterminated_string(span, self.closing))),
                put_back: LexerPutBack::None,
            }
        }
        // Escape next character
        else if next_char.value == '\\' {
            LexerStateTransition {
                new_state: Some(S::string(self.start, self.closing, true)),
                token_or_error: None,
                put_back: LexerPutBack::None,
            }
        } else {
            LexerStateTransition {
                new_state: None,
                token_or_error: None,
                put_back: LexerPutBack::None,
            }
        }
    }
}
