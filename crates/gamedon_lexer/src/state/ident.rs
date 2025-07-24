use crate::{
    GeneralError, GeneralKeyword, GeneralState, GeneralToken, LexerPutBack, LexerStateTransition,
    SourceChar, Span, WithSpan,
};

pub struct IdentState {
    pub start: usize,
}

impl IdentState {
    fn lex_ident<T: GeneralToken<K>, K: GeneralKeyword>(
        &self,
        text: &str,
        offset: usize,
    ) -> WithSpan<T> {
        assert!(self.start < offset);
        let span = Span {
            start: self.start,
            length: offset - self.start,
        };
        // SAFETY: Span originates from text meaning it should be within range.
        let lexeme = &text[span.range()];

        if let Some(keyword) = K::parse(lexeme) {
            T::keyword(span, keyword)
        } else {
            T::ident(span)
        }
    }

    #[inline]
    pub fn execute<S: GeneralState, T: GeneralToken<K>, E: GeneralError, K: GeneralKeyword>(
        &self,
        text: &str,
        next_char: Option<SourceChar>,
    ) -> LexerStateTransition<S, WithSpan<T>, WithSpan<E>> {
        let Some(next_char) = next_char else {
            let token = self.lex_ident(text, text.len());
            return LexerStateTransition {
                new_state: Some(S::finished()),
                token_or_error: Some(Ok(token)),
                put_back: LexerPutBack::None,
            };
        };

        if next_char.value.is_ascii_alphanumeric() || next_char.value == '_' {
            LexerStateTransition {
                new_state: None,
                token_or_error: None,
                put_back: LexerPutBack::None,
            }
        } else {
            let token = self.lex_ident(text, next_char.offset);
            LexerStateTransition {
                new_state: Some(S::initial(next_char.offset)),
                token_or_error: Some(Ok(token)),
                put_back: LexerPutBack::One([next_char]),
            }
        }
    }
}
