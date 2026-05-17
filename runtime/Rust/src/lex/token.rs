//! Symbols that parser works on
use std::borrow::{Borrow, Cow};

use std::fmt::Formatter;
use std::fmt::{Debug, Display};

use std::sync::atomic::{AtomicIsize, Ordering};

use crate::char_stream::InputData;
use crate::int_stream::EOF;
use crate::token_factory::{INVALID_COMMON, INVALID_OWNING};

use better_any::type_id;

/// Type of tokens that parser considers invalid
pub const TOKEN_INVALID_TYPE: i32 = 0;
/// Type of tokens that DFA can use to advance to next state without consuming actual input token.
/// Should not be created by downstream implementations.
pub const TOKEN_EPSILON: i32 = -2;
/// Min token type that can be assigned to tokens created by downstream implementations.
pub const TOKEN_MIN_USER_TOKEN_TYPE: i32 = 1;
/// Type of EOF token
pub const TOKEN_EOF: i32 = EOF;
/// Default channel lexer emits tokens to
pub const TOKEN_DEFAULT_CHANNEL: i32 = 0;
/// Predefined additional channel for lexer to assign tokens to
pub const TOKEN_HIDDEN_CHANNEL: i32 = 1;
/// Shorthand for TOKEN_HIDDEN_CHANNEL
pub const HIDDEN: i32 = TOKEN_HIDDEN_CHANNEL;

/// Implemented by tokens that are produced by a `TokenFactory`
#[allow(missing_docs)]
pub trait Token: Debug + Display {
    /// Type of the underlying data this token refers to
    type Data: ?Sized + InputData;
    // fn get_source(&self) -> Option<(Box<dyn TokenSource>, Box<dyn CharStream>)>;
    fn get_token_type(&self) -> i32;
    fn get_channel(&self) -> i32 {
        TOKEN_DEFAULT_CHANNEL
    }
    fn get_start(&self) -> isize {
        0
    }
    fn get_stop(&self) -> isize {
        0
    }
    fn get_line(&self) -> isize {
        0
    }
    fn get_column(&self) -> isize {
        0
    }

    fn get_text(&self) -> &Self::Data;
    fn set_text(&mut self, _text: <Self::Data as ToOwned>::Owned) {}

    fn get_token_index(&self) -> isize {
        0
    }
    fn set_token_index(&self, _v: isize) {}

    // fn get_token_source(&self) -> &dyn TokenSource;
    // fn get_input_stream(&self) -> &dyn CharStream;

    /// returns fully owned representation of this token
    fn to_owned(&self) -> OwningToken {
        OwningToken {
            token_type: self.get_token_type(),
            channel: self.get_channel(),
            start: self.get_start(),
            stop: self.get_stop(),
            token_index: AtomicIsize::from(self.get_token_index()),
            line: self.get_line(),
            column: self.get_column(),
            text: self.get_text().to_display(),
            read_only: true,
        }
    }
}

/// Token that owns its data
pub type OwningToken = GenericToken<String>;
/// Most versatile Token that uses Cow to save data
/// Can be used seamlessly switch from owned to zero-copy parsing
pub type CommonToken<'a> = GenericToken<Cow<'a, str>>;

type_id!(OwningToken);
type_id!(CommonToken<'a>);

#[derive(Debug)]
#[allow(missing_docs)]
pub struct GenericToken<T> {
    //    source: Option<(Box<TokenSource>,Box<CharStream>)>,
    pub token_type: i32,
    pub channel: i32,
    pub start: isize,
    pub stop: isize,
    pub token_index: AtomicIsize,
    pub line: isize,
    pub column: isize,
    pub text: T,
    pub read_only: bool,
}

impl<T: Clone> Clone for GenericToken<T>
where
    Self: Token,
{
    fn clone(&self) -> Self {
        Self {
            token_type: self.token_type,
            channel: self.channel,
            start: self.start,
            stop: self.stop,
            token_index: AtomicIsize::new(self.get_token_index()),
            line: self.line,
            column: self.column,
            text: self.text.clone(),
            read_only: false,
        }
    }
}

impl<T: Borrow<str> + Debug> Display for GenericToken<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let txt = if self.token_type == TOKEN_EOF {
            "<EOF>"
        } else {
            self.text.borrow()
        };
        let txt = txt.replace("\n", "\\n");
        let txt = txt.replace("\r", "\\r");
        let txt = txt.replace("\t", "\\t");
        //        let txt = escape_whitespaces(txt,false);
        f.write_fmt(format_args!(
            "[@{},{}:{}='{}',<{}>{},{}:{}]",
            self.get_token_index(),
            self.start,
            self.stop,
            txt,
            self.token_type,
            if self.channel > 0 {
                ",channel=".to_string() + self.channel.to_string().as_str()
            } else {
                String::new()
            },
            self.line,
            self.column
        ))
    }
}

impl<T: Borrow<str> + Debug> Token<T> {
    type Data = str;

    fn get_token_type(&self) -> i32 {
        self.token_type
    }

    fn get_channel(&self) -> i32 {
        self.channel
    }

    fn get_start(&self) -> isize {
        self.start
    }

    fn get_stop(&self) -> isize {
        self.stop
    }

    fn get_line(&self) -> isize {
        self.line
    }

    fn get_column(&self) -> isize {
        self.column
    }

    fn get_text(&self) -> &str {
        if self.token_type == EOF {
            "<EOF>"
        } else {
            self.text.borrow()
        }
    }

    fn set_text(&mut self, _text: String) {
        unimplemented!()
    }

    fn get_token_index(&self) -> isize {
        self.token_index.load(Ordering::Relaxed)
    }

    fn set_token_index(&self, _v: isize) {
        self.token_index.store(_v, Ordering::Relaxed)
    }

    fn to_owned(&self) -> OwningToken {
        OwningToken {
            token_type: self.token_type,
            channel: self.channel,
            start: self.start,
            stop: self.stop,
            token_index: AtomicIsize::new(self.get_token_index()),
            line: self.line,
            column: self.column,
            text: self.text.borrow().to_owned(),
            read_only: self.read_only,
        }
    }
}

impl Default for &'_ OwningToken {
    fn default() -> Self {
        &INVALID_OWNING
    }
}

impl Default for &'_ CommonToken<'_> {
    fn default() -> Self {
        &INVALID_COMMON
    }
}

//! `IntStream` that produces tokens for Parser
use std::borrow::Borrow;
use std::cmp::min;
use std::marker::PhantomData;

use crate::char_stream::InputData;
use crate::int_stream::{IntStream, IterWrapper};
use crate::token::{OwningToken, Token, TOKEN_EOF, TOKEN_INVALID_TYPE};
use crate::token_factory::TokenFactory;
use crate::token_source::TokenSource;
use std::fmt::{Debug, Formatter};

/// An `IntSteam` of `Token`s
///
/// Used as an input for `Parser`s
/// If there is an existing source of tokens, you should implement
/// `TokenSource`, not `TokenStream`
pub trait TokenStream<'input>: IntStream {
    /// Token factory that created tokens in this stream
    type TF: TokenFactory<'input> + 'input;

    /// Lookahead for tokens, same as `IntSteam::la` but return reference to full token
    fn lt(&mut self, k: isize) -> Option<&<Self::TF as TokenFactory<'input>>::Tok>;
    /// Returns reference to token at `index`
    fn get(&self, index: isize) -> &<Self::TF as TokenFactory<'input>>::Tok;

    /// Token source that produced data for tokens for this stream
    fn get_token_source(&self) -> &dyn TokenSource<'input, TF = Self::TF>;
    //    fn set_token_source(&self,source: Box<TokenSource>);
    /// Get combined text of all tokens in this stream
    fn get_all_text(&self) -> String {
        self.get_text_from_interval(0, self.size() - 1)
    }
    /// Get combined text of tokens in start..=stop interval
    fn get_text_from_interval(&self, start: isize, stop: isize) -> String;
    //    fn get_text_from_rule_context(&self,context: RuleContext) -> String;
    /// Get combined text of tokens in between `a` and `b`
    fn get_text_from_tokens<T: Token + ?Sized>(&self, a: &T, b: &T) -> String
    where
        Self: Sized,
    {
        self.get_text_from_interval(a.get_token_index(), b.get_token_index())
    }
}

/// Iterator over tokens in `T`
#[derive(Debug)]
pub struct TokenIter<'a, 'input: 'a, T: TokenStream<'input>>(
    &'a mut T,
    bool,
    PhantomData<fn() -> &'input str>,
);

impl<'a, 'input: 'a, T: TokenStream<'input>> Iterator for TokenIter<'a, 'input, T> {
    type Item = OwningToken;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 {
            return None;
        }
        let result = self.0.lt(1).unwrap().borrow().to_owned();
        if result.get_token_type() == TOKEN_EOF {
            self.1 = true;
        } else {
            self.0.consume();
        }
        Some(result)
    }
}

/// Token stream that keeps all data in internal Vec
pub struct UnbufferedTokenStream<'input, T: TokenSource<'input>> {
    token_source: T,
    pub(crate) tokens: Vec<<T::TF as TokenFactory<'input>>::Tok>,
    //todo prev token for lt(-1)
    pub(crate) current_token_index: isize,
    markers_count: isize,
    pub(crate) p: isize,
    fetched_eof: bool,
}

impl<'input, T: TokenSource<'input>> Debug for UnbufferedTokenStream<'input, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnbufferedTokenStream")
            .field("tokens", &self.tokens)
            .field("current_token_index", &self.current_token_index)
            .field("markers_count", &self.markers_count)
            .field("p(buffer index)", &self.p)
            .finish()
    }
}

impl<'input, T: TokenSource<'input>> UnbufferedTokenStream<'input, T> {
    /// Creates iterator over this token stream
    pub fn iter(&mut self) -> IterWrapper<'_, Self> {
        IterWrapper(self, false)
    }

    /// Creates iterator over tokens in this token stream
    pub fn token_iter(&mut self) -> TokenIter<'_, 'input, Self> {
        TokenIter(self, false, PhantomData)
    }

    /// Creates token stream that keeps all tokens inside
    pub fn new_buffered(source: T) -> Self {
        let mut a = UnbufferedTokenStream::new_unbuffered(source);
        a.mark();
        a
    }

    /// Creates token stream that keeps only tokens required by `mark`
    pub fn new_unbuffered(source: T) -> Self {
        UnbufferedTokenStream {
            token_source: source,
            tokens: vec![],
            current_token_index: 0,
            markers_count: 0,
            p: 0,
            fetched_eof: false,
        }
    }

    pub fn get_dfa_string(&self) -> String {
        self.token_source.get_dfa_string()
    }

    fn sync(&mut self, want: isize) {
        let need = (self.p + want - 1) - self.tokens.len() as isize + 1;
        if need > 0 {
            self.fill(need);
        }
    }

    fn get_buffer_start_index(&self) -> isize {
        self.current_token_index - self.p
    }

    pub(crate) fn fill(&mut self, need: isize) -> isize {
        for i in 0..need {
            if !self.tokens.is_empty()
                && self.tokens.last().unwrap().borrow().get_token_type() == TOKEN_EOF
            {
                return i;
            }
            let token = self.token_source.next_token();
            token
                .borrow()
                .set_token_index(self.get_buffer_start_index() + self.tokens.len() as isize);
            self.tokens.push(token);
        }

        need
    }
}

impl<'input, T: TokenSource<'input>> TokenStream<'input> for UnbufferedTokenStream<'input, T> {
    type TF = T::TF;

    #[inline]
    fn lt(&mut self, i: isize) -> Option<&<Self::TF as TokenFactory<'input>>::Tok> {
        if i == -1 {
            return self.tokens.get(self.p as usize - 1);
        }

        self.sync(i);

        self.tokens.get((self.p + i - 1) as usize)
    }

    #[inline]
    fn get(&self, index: isize) -> &<Self::TF as TokenFactory<'input>>::Tok {
        &self.tokens[(index - self.get_buffer_start_index()) as usize]
    }

    fn get_token_source(&self) -> &dyn TokenSource<'input, TF = Self::TF> {
        &self.token_source
    }

    fn get_text_from_interval(&self, start: isize, stop: isize) -> String {
        //        println!("get_text_from_interval {}..{}",start,stop);
        //        println!("all tokens {:?}",self.tokens.iter().map(|x|x.as_ref().to_owned()).collect::<Vec<OwningToken>>());

        let buffer_start_index = self.get_buffer_start_index();
        let buffer_stop_index = buffer_start_index + self.tokens.len() as isize - 1;
        if start < buffer_start_index || stop > buffer_stop_index {
            panic!(
                "interval {}..={} not in token buffer window: {}..{}",
                start, stop, buffer_start_index, buffer_stop_index
            );
        }

        let a = start - buffer_start_index;
        let b = stop - buffer_start_index;

        let mut buf = String::new();
        for i in a..(b + 1) {
            let t = self.tokens[i as usize].borrow();
            if t.get_token_type() == TOKEN_EOF {
                break;
            }
            buf.push_str(&t.get_text().to_display());
        }

        buf
    }
}

impl<'input, T: TokenSource<'input>> IntStream for UnbufferedTokenStream<'input, T> {
    #[inline]
    fn consume(&mut self) {
        if self.fetched_eof {
            panic!("cannot consume EOF");
        }
        if self.la(1) == TOKEN_EOF {
            self.fetched_eof = true;
        }

        if self.p == self.tokens.len() as isize && self.markers_count == 0 {
            self.tokens.clear();
            self.p = -1;
        }

        self.p += 1;
        self.current_token_index += 1;

        self.sync(1);
        // Ok(())
    }

    #[inline]
    fn la(&mut self, i: isize) -> i32 {
        self.lt(i)
            .map(|t| t.borrow().get_token_type())
            .unwrap_or(TOKEN_INVALID_TYPE)
    }

    #[inline]
    fn mark(&mut self) -> isize {
        self.markers_count += 1;
        -self.markers_count
    }

    #[inline]
    fn release(&mut self, marker: isize) {
        assert_eq!(marker, -self.markers_count);

        self.markers_count -= 1;
        if self.markers_count == 0 && self.p > 0 {
            self.tokens.drain(0..self.p as usize);

            self.p = 0;
        }
    }

    #[inline(always)]
    fn index(&self) -> isize {
        self.current_token_index
    }

    #[inline]
    fn seek(&mut self, mut index: isize) {
        if self.current_token_index == index {
            return;
        }
        if index > self.current_token_index {
            self.sync(index - self.current_token_index);
            index = min(index, self.get_buffer_start_index() + self.size() + 1);
        }
        let i = index - self.get_buffer_start_index();
        if i < 0 || i >= self.tokens.len() as isize {
            panic!()
        }

        self.p = i;
        self.current_token_index = index;
    }

    #[inline(always)]
    fn size(&self) -> isize {
        self.tokens.len() as isize
    }

    fn get_source_name(&self) -> String {
        self.token_source.get_source_name()
    }
}
