//! Lexer implementation

use std::{cell::LazyCell, collections::VecDeque, iter::Peekable, marker::PhantomData, str::Chars};

use crate::{atn::ATN, lex::{token::{TokenChannel, Token, TokenType}}};

pub trait Lex {
    const CHANNEL_NAMES: LazyCell<Vec<&'static str>>;
    const MODE_NAMES: LazyCell<Vec<&'static str>>;
    const RULE_NAMES: LazyCell<Vec<&'static str>>;
    const LITERAL_NAMES: LazyCell<Vec<Option<&'static str>>>;
}

#[derive(Debug)]
struct LexerPosition {
    absolute: usize,
    line: usize,
    char_position_in_line: usize,
}

impl LexerPosition {
    pub fn new() -> LexerPosition {
        Self {
            absolute: 0,
            line: 0,
            char_position_in_line: 0
        }
    }

    pub fn next(&mut self, c: char) {
        self.absolute += 1;

        if c != '\n' {
            self.char_position_in_line += 1;
        } else {
            self.line += 1;
            self.char_position_in_line = 0
        }
    }

    pub fn absolute(&self) -> usize {
        self.absolute
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn char_position_in_line(&self) -> usize {
        self.char_position_in_line
    }
}

/// Default implementation of Lexer
///
/// Public fields in this struct are intended to be used by embedded actions
#[allow(missing_docs)]
pub struct Lexer<'input, L: Lex> {
    _p: PhantomData<L>,
    atn: ATN,
    input: Vec<char>,
    
    pos: LexerPosition,

    mode_stack: VecDeque<LexerMode>,
    mode: LexerMode,

    token: Option<Token<'input>>,
}

pub enum LexerMode {
    Default,

    /// Special token type to indicate that lexer should continue current token on next iteration
    More,

    /// Special token type to indicate that lexer should not return current token
    /// usually used to skip whitespaces and comments
    Skip
}

pub(crate) const LEXER_MIN_CHAR_VALUE: usize = 0x0000;
pub(crate) const LEXER_MAX_CHAR_VALUE: usize = 0x10FFFF;

impl<'input, L: Lex> Lexer<'input, L>
{
    /// Creates new lexer instance
    pub fn new(
        input: &'input str,
        atn: ATN
    ) -> Self {
        Self {
            _p: PhantomData,

            atn,
            input: input.chars().collect(),

            pos: LexerPosition::new(),

            mode_stack: VecDeque::new(),
            mode: LexerMode::Default,

            token: None
        }
    }
    
    fn look_ahead(&self, by: usize) -> Option<char> {
        self.input.get(self.pos.absolute + by).copied()
    }

    pub fn emit(&mut self) -> Option<Token<'_>> {
        self.token.take()
    }
}