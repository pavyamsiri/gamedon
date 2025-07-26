use core::{error, fmt};
use std::{collections::VecDeque, ops::Range, str::CharIndices};

/// Module of lexer states.
pub mod state;

/// A trait used to define a general lexer state for use when creating reusable lexing procedures.
pub trait GeneralState {
    /// Create a initial/normal state at byte offset `start`.
    fn initial(start: usize) -> Self;
    /// Create a string lexing state at byte offset `start` using delimiter `closing`.
    /// The `should_escape` flag signifies to the state machine whether to escape the closing character.
    fn string(start: usize, closing: char, should_escape: bool) -> Self;
    /// Create the finished state.
    fn finished() -> Self;
}

/// Trait to be implemented by token types to indicate whether a given token is an EOF token.
pub trait TokenWithEnd {
    /// Check if the given token is EOF.
    fn is_eof(&self) -> bool;
}

impl<T: TokenWithEnd> TokenWithEnd for WithSpan<T> {
    fn is_eof(&self) -> bool {
        self.kind.is_eof()
    }
}

/// A trait used to define a general token type and functions to create certain tokens.
pub trait GeneralToken<K>: Sized + TokenWithEnd {
    /// Create an EOF token at `offset` in bytes.
    fn eof(offset: usize) -> WithSpan<Self>;
    /// Create a string token with the given span.
    fn string(span: Span) -> WithSpan<Self>;
    /// Create a identifier token with the given span.
    fn ident(span: Span) -> WithSpan<Self>;
    /// Create a keyword token with the given span.
    fn keyword(span: Span, keyword: K) -> WithSpan<Self>;
}

/// A trait to define a general lexer error type.
pub trait GeneralError: Sized {
    /// Create an unterminated string lexer error.
    fn unterminated_string(span: Span, delimiter: char) -> WithSpan<Self>;
}

/// A trait to define a general keyword token type.
pub trait GeneralKeyword: Sized {
    /// Parse a string to a keyword if it is a valid keyword.
    fn parse(lexeme: &str) -> Option<Self>;
}

/// A span of text in source defined by its starting byte offset and its length in bytes.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Span {
    /// The byte offset to the start of the span.
    pub start: usize,
    /// The length in bytes of the span.
    pub length: usize,
}

impl Span {
    /// Return the range.
    #[inline]
    pub const fn range(&self) -> Range<usize> {
        self.start..(self.start + self.length)
    }
}

/// A wrapper type around a `Kind` type (usually an enum) and its associated span.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WithSpan<Kind> {
    /// The kind.
    kind: Kind,
    /// The span.
    span: Span,
}

impl<T> WithSpan<T> {
    /// Return the kind as a reference.
    #[inline]
    pub const fn get_kind(&self) -> &T {
        &self.kind
    }

    /// Consume the wrapper and return the inner kind.
    #[inline]
    pub fn take_kind(self) -> T {
        self.kind
    }

    /// Return the span.
    #[inline]
    pub const fn get_span(&self) -> &Span {
        &self.span
    }

    /// Consume the wrapper into its kind and span.
    #[inline]
    pub fn take(self) -> (T, Span) {
        (self.kind, self.span)
    }

    /// Transform the kind from `T` to `U`.
    #[inline]
    pub fn map_kind<U>(self, func: impl FnOnce(T) -> U) -> U {
        func(self.kind)
    }
}

impl<T: fmt::Display> fmt::Display for WithSpan<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}..{}]: {}",
            self.span.start,
            self.span.start + self.span.length,
            self.kind
        )
    }
}

impl<T: error::Error> error::Error for WithSpan<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.kind.source()
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        self.source()
    }
}

/// Wrap a type with a span.
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

/// Represents a source character.
#[derive(Debug, Clone, Copy)]
pub struct SourceChar {
    /// The character itself.
    pub value: char,
    /// The byte offset of the character in the source.
    pub offset: usize,
}

impl SourceChar {
    /// Calculates the offset of the next character in the source assuming UTF-8 encoding.
    #[inline]
    pub const fn next_offset(&self) -> usize {
        self.offset + self.value.len_utf8()
    }
}

/// An iterator over a source text to be used with lexers.
struct SourceLookup<'src> {
    /// The source text.
    text: &'src str,
    /// An iterator over the characters.
    chars: CharIndices<'src>,
    /// A lookahead cache used to backtrack.
    lookahead: VecDeque<SourceChar>,
}

impl<'src> SourceLookup<'src> {
    /// Create from a source string.
    fn new(source: &'src str) -> Self {
        Self {
            text: source,
            chars: source.char_indices(),
            lookahead: VecDeque::new(),
        }
    }

    /// Return the next character.
    fn next_char(&mut self) -> Option<SourceChar> {
        match self.lookahead.pop_front() {
            Some(s) => Some(s),
            None => {
                let (offset, value) = self.chars.next()?;
                Some(SourceChar { value, offset })
            }
        }
    }

    /// Populate the lookahead buffer.
    fn put_back(&mut self, chars: &[SourceChar]) {
        self.lookahead.extend(chars);
    }

    /// Return the source.
    const fn get_text(&self) -> &'src str {
        self.text
    }

    /// Return the lexeme associated with the given span if the span is valid.
    fn get_lexeme(&self, span: &Span) -> Option<&'src str> {
        let range = span.range();
        if range.end > self.text.len() {
            return None;
        }
        Some(&self.text[range])
    }
}

/// The characters to put back into the lexer in cases where the state machine needs to backtrack.
pub enum LexerPutBack {
    /// No characters to put back.
    None,
    /// One character to put back.
    One([SourceChar; 1]),
    /// Two characters to put back.
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

/// Represents a state transition in the lexer.
pub struct LexerStateTransition<S, T, E> {
    /// The new state to transition to unless it is `None` then no state change will take place.
    pub new_state: Option<S>,
    /// A token or an error if it is lexed.
    pub token_or_error: Option<Result<T, E>>,
    /// The characters to put back.
    pub put_back: LexerPutBack,
}

pub trait LexerState {
    /// The token type.
    type Token;
    /// The error type.
    type Error;

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

impl<S> Lexer<'_, S>
where
    S: LexerState,
    S::Token: Clone,
    S::Error: Clone,
{
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
}

impl<'src, S> IntoIterator for Lexer<'src, S>
where
    S: LexerState,
    S::Token: TokenWithEnd,
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

/// Iterator over tokens.
pub struct LexerIterator<'src, S: LexerState> {
    lexer: Lexer<'src, S>,
    is_eof: bool,
}

impl<S> Iterator for LexerIterator<'_, S>
where
    S: LexerState,
    S::Token: TokenWithEnd,
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
    use core::{
        cmp,
        fmt::{self, Write},
    };

    type TokenStream<T, E> = [Result<WithSpan<T>, WithSpan<E>>];

    /// Assert that two token streams are the same, otherwise print the difference.
    #[inline]
    pub fn assert_token_stream_eq<T, E>(
        input: &str,
        actual_stream: &TokenStream<T, E>,
        expected_stream: &TokenStream<T, E>,
    ) where
        T: PartialEq + fmt::Debug,
        E: PartialEq + fmt::Debug,
    {
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
            cmp::Ordering::Less => {
                writeln!(buffer, "Expected tokens has extra tokens!").unwrap();
                Some(expected_stream.iter().enumerate().skip(actual_stream.len()))
            }
            cmp::Ordering::Equal => None,
            cmp::Ordering::Greater => {
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
