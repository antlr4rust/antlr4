use std::{cell::LazyCell, marker::PhantomData};

use crate::{atn::{ATN, ATNRuleRef}, lex::{Lex, Lexer}};

pub trait Parse {
    const RULE_NAMES: LazyCell<Vec<&'static str>>;
    const LITERAL_NAMES: LazyCell<Vec<Option<&'static str>>>;
    // const SYMBOLIC_NAMES: LazyCell<Vec<Option<&'static str>>>;
    const ATN: LazyCell<ATN>;
}

pub struct Parser<T: Parse> {
    _p: PhantomData<T>,
    atn: ATN
}

impl<T: Parse> Parser<T> {
    pub fn new<L: Lex>(lexer: Lexer<L>) -> Self { 
        Self {
            _p: PhantomData,

            atn: LazyCell::into_inner(T::ATN).unwrap(),
        }
    }

    pub fn enter_rule(&mut self, rule: ATNRuleRef) {
        self.atn.enter_rule(rule);
    }
}

// Given this grammar 
// program
//     : a EOF
//     | b EOF
//     | mult EOF
//     ;

// mult: b c ;

// a: 'a' ;
// b: 'bb' ;
// c: 'c'* ;

// An impl would look like this (based on the older generated version)?
// impl Parser<Grammar> {
//     pub fn program(&mut self) -> Result<Program, ANTLRError> {
//         self.atn.set_head(17);
//         let a = self.a()?; // self.a() is defined elsewhere
//         self.atn.set_head(11);
//     }
// }