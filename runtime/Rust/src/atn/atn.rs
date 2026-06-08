use std::collections::VecDeque;
use std::slice::Iter;

use std::fmt::{Debug, Formatter};

use crate::atn::ATNRuleRef;
use crate::atn::parse::read_edges;
use crate::atn::rule::ATNRule;
use crate::atn::state::ATNState;
use crate::atn::transition::Transition;
use crate::lex::{LexerAction};
use crate::lex::token::TokenType;
#[doc(hidden)]
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ATNType {
    LEXER = 0,
    PARSER,
}

impl ATNType {
    pub fn new(i: usize) -> Result<ATNType, ()> {
        match i {
            0 => Ok(Self::LEXER),
            1 => Ok(Self::PARSER),
            _ => Err(()),
        }
    }
}

pub const INVALID_ALT: usize = 0;

enum ATNConstructionErr {
    VersionMismatch,
    InsufficientData,
    InvalidGrammarType,
    ReadEdges
}
/// Augmented Transition Network
///
/// Basically NFA(graph) of states and possible(maybe multiple) transitions on a given particular symbol.
///
pub struct ATN {
    grammar_type: ATNType,

    max_token_type: usize,
    lexer_actions: Vec<LexerAction>,

    pub(super) rules: Vec<ATNRule>,
    pub(super) rule_stack: VecDeque<ATNRuleRef>,

    pub(super) states: Vec<ATNState>,
    pub(super) transitions: Vec<Transition>
}

impl Debug for ATN {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ATN")
            .field("grammar_type", &self.grammar_type)
            .field("max_token_type", &self.max_token_type)
            .field("states count", &self.states.len())
            .field("..", &"..")
            .finish()
    }
}

impl ATN {
    const SERIALIZED_VERSION: usize = 4;

    pub fn from_serialized(data: &mut Iter<usize>, verify_atn: bool) -> Result<Self, ATNConstructionErr> {
        let version = *data.next().ok_or(ATNConstructionErr::InsufficientData)?;
        if version != Self::SERIALIZED_VERSION {
            return Err(ATNConstructionErr::VersionMismatch);
        };

        let grammar_type = *data.next().ok_or(ATNConstructionErr::InsufficientData)?;
        let grammar_type = ATNType::new(grammar_type).map_err(|_| ATNConstructionErr::InvalidGrammarType)?;

        let max_token_type = *data.next().ok_or(ATNConstructionErr::InsufficientData)?;

        let mut in_construction_atn = ATN {
            grammar_type,
            lexer_actions: Vec::new(),
            max_token_type,

            rules: Vec::new(),
            rule_stack: VecDeque::new(),
            states: Vec::new(),
            transitions: Vec::new()
        };

        in_construction_atn.transitions = read_edges(data).ok_or(ATNConstructionErr::ReadEdges)?;
 
        Ok(in_construction_atn)
    }

    pub fn atn_type(&self) -> ATNType {
        self.grammar_type
    }

}