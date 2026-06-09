use crate::atn::ATNStateRef;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub enum ATNRule {
    Lexer {
        start_state: ATNStateRef,
        stop_state: ATNStateRef,
        token_type: usize
    },
    
    Parser {
        start_state: ATNStateRef,
        stop_state: ATNStateRef,
    }
}