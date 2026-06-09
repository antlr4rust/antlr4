use std::collections::VecDeque;
use std::slice::Iter;

use std::fmt::{Debug, Formatter};

use crate::atn::ATNRuleRef;
use crate::atn::parse::{read_edges, read_modes, read_rules, read_sets, read_states};
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

#[derive(Debug)]
pub enum ATNConstructionErr {
    VersionMismatch,
    InsufficientData,
    InvalidGrammarType,
    ReadStates,
    ReadRules,
    ReadEdges,
    ReadModes,
    ReadSets
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
            .field("rules", &self.rules)
            .field("states", &self.states)
            .field("transitions", &self.transitions)
            .finish()
    }
}

impl ATN {
    const SERIALIZED_VERSION: usize = 4;

    pub fn from_serialized(data: Vec<usize>, verify_atn: bool) -> Result<Self, ATNConstructionErr> {
        let mut data = data.iter();
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

        in_construction_atn.states = read_states(&mut data).ok_or(ATNConstructionErr::ReadStates)?;
        in_construction_atn.rules = read_rules(&mut data, false).ok_or(ATNConstructionErr::ReadRules)?;
        read_modes(&mut data).ok_or(ATNConstructionErr::ReadModes)?;
        read_sets(&mut data).ok_or(ATNConstructionErr::ReadSets)?;
        
        in_construction_atn.transitions = read_edges(&mut data).ok_or(ATNConstructionErr::ReadEdges)?;
 
        // read_decisions
        // read_lexer_actions (if lexer)
        // mark_precedence_decisions

        Ok(in_construction_atn)
    }

    pub fn atn_type(&self) -> ATNType {
        self.grammar_type
    }

}