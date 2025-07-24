use std::{collections::VecDeque, ops::Range, str::CharIndices};
pub mod state;

pub trait GeneralState {
    fn initial(start: usize) -> Self;
    fn string(start: usize, closing: char, should_escape: bool) -> Self;
    fn finished() -> Self;
}

pub trait TokenWithEnd {
    fn is_eof(&self) -> bool;
}

pub trait GeneralToken<K: GeneralKeyword>: Sized + TokenWithEnd {
    fn eof(offset: usize) -> WithSpan<Self>;
    fn string(span: Span) -> WithSpan<Self>;
    fn ident(span: Span) -> WithSpan<Self>;
    fn keyword(span: Span, keyword: K) -> WithSpan<Self>;
}

pub trait GeneralError: Sized {
    fn unterminated_string(span: Span, delimiter: char) -> WithSpan<Self>;
}

pub trait GeneralKeyword: Sized {
    fn parse(lexeme: &str) -> Option<Self>;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Span {
    /// The byte offset to the start of the span.
    pub start: usize,
    /// The length in bytes of the span.
    pub length: usize,
}

impl Span {
    pub const fn range(&self) -> Range<usize> {
        self.start..(self.start + self.length)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WithSpan<Kind> {
    /// The kind.
    kind: Kind,
    /// The span.
    span: Span,
}

impl<T> WithSpan<T> {
    #[inline]
    pub const fn get_kind(&self) -> &T {
        &self.kind
    }

    #[inline]
    pub const fn get_span(&self) -> &Span {
        &self.span
    }

    #[inline]
    pub fn take_kind(self) -> T {
        self.kind
    }

    #[inline]
    pub fn map_kind<U>(self, func: impl FnOnce(T) -> U) -> U {
        func(self.kind)
    }

    #[inline]
    pub fn take(self) -> (T, Span) {
        (self.kind, self.span)
    }
}

impl<T: TokenWithEnd> TokenWithEnd for WithSpan<T> {
    fn is_eof(&self) -> bool {
        self.kind.is_eof()
    }
}

impl<T: std::fmt::Display> std::fmt::Display for WithSpan<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}..{}]: {}",
            self.span.start,
            self.span.start + self.span.length,
            self.kind
        )
    }
}

impl<T: std::error::Error> std::error::Error for WithSpan<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.kind.source()
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

/// Wrap a kind with a span.
pub trait WrapWithSpan {
    /// Wrap self with a span.
    fn wrap(self, span: Span) -> WithSpan<Self>
    where
        Self: Sized;

    /// Inherit a span.
    fn inherit<T>(self, parent: WithSpan<T>) -> WithSpan<Self>
    where
        Self: Sized;
}

impl<T> WrapWithSpan for T {
    fn wrap(self, span: Span) -> WithSpan<Self>
    where
        Self: Sized,
    {
        WithSpan { kind: self, span }
    }

    fn inherit<U>(self, parent: WithSpan<U>) -> WithSpan<Self>
    where
        Self: Sized,
    {
        WithSpan {
            kind: self,
            span: parent.span,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SourceChar {
    pub value: char,
    pub offset: usize,
}

impl SourceChar {
    #[inline]
    pub const fn next_offset(&self) -> usize {
        self.offset + self.value.len_utf8()
    }
}

pub struct SourceLookup<'src> {
    /// The source text.
    text: &'src str,
    /// An iterator over the characters.
    chars: CharIndices<'src>,
    /// A lookahead cache used to backtrack.
    lookahead: VecDeque<SourceChar>,
}

impl<'src> SourceLookup<'src> {
    #[inline]
    pub fn new(source: &'src str) -> Self {
        assert!(
            source.len() < u32::MAX as usize,
            "Lexers do not support files with more than {} bytes.",
            u32::MAX - 1,
        );
        Self {
            text: source,
            chars: source.char_indices(),
            lookahead: VecDeque::new(),
        }
    }

    /// Return the next character.
    #[inline]
    pub fn next_char(&mut self) -> Option<SourceChar> {
        match self.lookahead.pop_front() {
            Some(s) => Some(s),
            None => {
                let (offset, value) = self.chars.next()?;
                Some(SourceChar { value, offset })
            }
        }
    }

    /// Populate the lookahead buffer.
    #[inline]
    pub fn put_back(&mut self, chars: &[SourceChar]) {
        self.lookahead.extend(chars);
    }

    /// Return the source.
    pub const fn get_text(&self) -> &'src str {
        self.text
    }

    /// Return the lexeme associated with the given span if the span is valid.
    #[inline]
    pub fn get_lexeme(&self, span: &Span) -> Option<&'src str> {
        let range = span.range();
        if range.end > self.text.len() {
            return None;
        }
        Some(&self.text[range])
    }
}

pub enum LexerPutBack {
    None,
    One([SourceChar; 1]),
    Two([SourceChar; 2]),
}

impl LexerPutBack {
    /// Return a slice of characters.
    pub const fn to_chars(&self) -> &[SourceChar] {
        match self {
            LexerPutBack::None => &[],
            LexerPutBack::One(chars) => chars,
            LexerPutBack::Two(chars) => chars,
        }
    }
}

pub struct LexerStateTransition<S, T, E> {
    /// The new state to transition.
    pub new_state: Option<S>,
    /// The lexed token or an error.
    pub token_or_error: Option<Result<T, E>>,
    /// The characters to put back.
    pub put_back: LexerPutBack,
}

pub trait LexerState {
    /// The token type.
    type Token: TokenWithEnd + Clone;
    /// The error type.
    type Error: Clone;
    type Keyword: GeneralKeyword;

    fn initial() -> Self;

    fn execute(
        &self,
        text: &str,
        next_char: Option<SourceChar>,
    ) -> LexerStateTransition<Self, Self::Token, Self::Error>
    where
        Self: Sized;
}

pub struct Lexer<'src, S: LexerState> {
    /// The source text.
    source: SourceLookup<'src>,
    /// The lexer state.
    state: S,
    /// Lookahead token.
    lookahead: Option<Result<S::Token, S::Error>>,
}

impl<'src, S: LexerState> Lexer<'src, S> {
    #[inline]
    pub fn get_lexeme(&self, span: &Span) -> Option<&'src str> {
        self.source.get_lexeme(span)
    }
}

impl<'src, S> Lexer<'src, S>
where
    S: LexerState,
{
    #[inline]
    pub fn new(source: &'src str) -> Self {
        Self {
            source: SourceLookup::new(source),
            state: S::initial(),
            lookahead: None,
        }
    }

    /// Lex the next token.
    #[inline]
    pub fn peek_token(&mut self) -> Result<S::Token, S::Error> {
        match &self.lookahead {
            Some(next_token) => next_token.clone(),
            None => {
                let next_token = self.next_token_impl();
                self.lookahead = Some(next_token.clone());
                next_token
            }
        }
    }

    /// Lex the next token.
    #[inline]
    pub fn next_token(&mut self) -> Result<S::Token, S::Error> {
        if let Some(next_token) = self.lookahead.take() {
            return next_token;
        }

        self.next_token_impl()
    }

    /// Lex the next token ignoring peeked token.
    fn next_token_impl(&mut self) -> Result<S::Token, S::Error> {
        loop {
            let next_char = self.source.next_char();
            let LexerStateTransition {
                new_state,
                token_or_error,
                put_back,
            } = self.state.execute(self.source.get_text(), next_char);

            // Change state
            if let Some(new_state) = new_state {
                self.state = new_state;
            }

            // Put back
            self.source.put_back(put_back.to_chars());

            // Return token or error
            if let Some(token_or_error) = token_or_error {
                return token_or_error;
            }
        }
    }
}

impl<'src, S> IntoIterator for Lexer<'src, S>
where
    S: LexerState,
{
    type Item = Result<S::Token, S::Error>;
    type IntoIter = LexerIterator<'src, S>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            lexer: self,
            is_eof: false,
        }
    }
}

pub struct LexerIterator<'src, S: LexerState> {
    lexer: Lexer<'src, S>,
    is_eof: bool,
}

impl<S> Iterator for LexerIterator<'_, S>
where
    S: LexerState,
{
    type Item = Result<S::Token, S::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.lexer.next_token();

        if let Ok(tok) = &res {
            if tok.is_eof() {
                if self.is_eof {
                    return None;
                }
                self.is_eof = true;
            }
        }
        Some(res)
    }
}

#[cfg(test)]
mod tests {}

pub mod test_utils {
    use crate::WithSpan;

    type TokenStream<T, E> = [Result<WithSpan<T>, WithSpan<E>>];
    #[inline]
    pub fn assert_token_stream_eq<T, E>(
        input: &str,
        actual_stream: &TokenStream<T, E>,
        expected_stream: &TokenStream<T, E>,
    ) where
        T: PartialEq + std::fmt::Debug,
        E: PartialEq + std::fmt::Debug,
    {
        use std::fmt::Write;

        if actual_stream == expected_stream {
            return;
        }

        let mut buffer = String::from("\nToken streams are not the same!\n");
        // Check if they have the same length
        for (index, (actual, expected)) in
            actual_stream.iter().zip(expected_stream.iter()).enumerate()
        {
            match (actual, expected) {
                (Ok(left), Ok(right)) => {
                    if left == right {
                        continue;
                    }
                    writeln!(
                        buffer,
                        "[{index:03}]: Token::{:?} ({:?}) != Token::{:?} ({:?})",
                        left.get_kind(),
                        left.get_span().range(),
                        right.get_kind(),
                        right.get_span().range(),
                    )
                    .unwrap();
                    writeln!(
                        buffer,
                        "\t{} vs {}",
                        &input[left.get_span().range()],
                        &input[right.get_span().range()]
                    )
                    .unwrap();
                }
                (Err(left), Err(right)) => {
                    if left == right {
                        continue;
                    }
                    writeln!(
                        buffer,
                        "[{index:03}]: Error::{:?} ({:?}) != Error::{:?} ({:?})",
                        left.get_kind(),
                        left.get_span().range(),
                        right.get_kind(),
                        right.get_span().range(),
                    )
                    .unwrap();
                    writeln!(
                        buffer,
                        "\t{} vs {}",
                        &input[left.get_span().range()],
                        &input[right.get_span().range()]
                    )
                    .unwrap();
                }
                (Ok(left), Err(right)) => {
                    writeln!(
                        buffer,
                        "[{index:03}]: Token::{:?} ({:?}) != Error::{:?} ({:?})",
                        left.get_kind(),
                        left.get_span().range(),
                        right.get_kind(),
                        right.get_span().range(),
                    )
                    .unwrap();
                    writeln!(
                        buffer,
                        "\t{} vs {}",
                        &input[left.get_span().range()],
                        &input[right.get_span().range()]
                    )
                    .unwrap();
                }
                (Err(left), Ok(right)) => {
                    writeln!(
                        buffer,
                        "[{index:03}]: Error::{:?} ({:?}) != Token::{:?} ({:?})",
                        left.get_kind(),
                        left.get_span().range(),
                        right.get_kind(),
                        right.get_span().range(),
                    )
                    .unwrap();
                    writeln!(
                        buffer,
                        "\t{} vs {}",
                        &input[left.get_span().range()],
                        &input[right.get_span().range()]
                    )
                    .unwrap();
                }
            }
        }

        let rest = match actual_stream.len().cmp(&expected_stream.len()) {
            std::cmp::Ordering::Less => {
                writeln!(buffer, "Expected tokens has extra tokens!").unwrap();
                Some(expected_stream.iter().enumerate().skip(actual_stream.len()))
            }
            std::cmp::Ordering::Equal => None,
            std::cmp::Ordering::Greater => {
                writeln!(buffer, "Actual tokens has extra tokens!").unwrap();
                Some(actual_stream.iter().enumerate().skip(expected_stream.len()))
            }
        };

        if let Some(rest) = rest {
            for (index, extra) in rest {
                match extra {
                    Ok(inner) => {
                        writeln!(
                            buffer,
                            "[{index:03}]: Token::{:?} ({:?})",
                            inner.get_kind(),
                            inner.get_span().range(),
                        )
                        .unwrap();
                        writeln!(buffer, "\t\"{}\"", &input[inner.get_span().range()],).unwrap();
                    }
                    Err(inner) => {
                        writeln!(
                            buffer,
                            "[{index:03}]: Error::{:?} ({:?})",
                            inner.get_kind(),
                            inner.get_span().range(),
                        )
                        .unwrap();
                        writeln!(buffer, "\t\"{}\"", &input[inner.get_span().range()],).unwrap();
                    }
                }
            }
        }

        panic!("{}", buffer);
    }
}
