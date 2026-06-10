use std::collections::{HashSet, VecDeque};
use std::slice::Iter;

use std::fmt::{Debug, Formatter};

use crate::atn::state::ATNStateType;
use crate::atn::{ATNRuleRef, ATNStateRef};
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

    pub(super) states: Vec<ATNState>,
    pub(super) rules: Vec<ATNRule>,
    pub(super) rule_stack: VecDeque<ATNRuleRef>,
    pub(super) modes: Vec<ATNStateRef>,
    pub(super) sets: Vec<HashSet<usize>>,
    pub(super) transitions: Vec<Transition>,
}

impl Debug for ATN {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ATN")
            .field("grammar_type", &self.grammar_type)
            .field("max_token_type", &self.max_token_type)
            
            .field("states", &self.states)
            .field("rules", &self.rules)
            .field("modes", &self.modes)
            .field("sets", &self.sets)
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


        let states = read_states(&mut data).ok_or(ATNConstructionErr::ReadStates)?;
        let mut rules = read_rules(&mut data, false).ok_or(ATNConstructionErr::ReadRules)?;
        for state in states.iter() {
            if state.state_type == ATNStateType::RuleStopState {
                if let Some(
                    ATNRule::Lexer { stop_state, .. }
                    | ATNRule::Parser { stop_state, .. }
                ) = rules.get_mut(state.rule_index) {
                    *stop_state = state.state_number
                }
            }
        }
        
        let modes = read_modes(&mut data).ok_or(ATNConstructionErr::ReadModes)?;
        let sets = read_sets(&mut data).ok_or(ATNConstructionErr::ReadSets)?;
        let edges = read_edges(data).ok_or(ATNConstructionErr::ReadEdges)?;
        
        Ok(ATN {
            grammar_type,
            max_token_type,
            lexer_actions: Vec::new(),
            states,
            rules,
            rule_stack: VecDeque::new(),
            modes,
            sets,
            transitions: edges,
        })
    }

    pub fn atn_type(&self) -> ATNType {
        self.grammar_type
    }

}