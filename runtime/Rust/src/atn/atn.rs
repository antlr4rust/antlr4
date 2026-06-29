use std::collections::{HashSet, VecDeque};
use std::slice::Iter;

use std::fmt::{Debug, Formatter};

use crate::atn::state::ATNStateType;
use crate::atn::{ATNRuleRef, ATNStateRef};
use crate::atn::parse::{read_decisions, read_edges, read_lex_actions, read_modes, read_rules, read_sets, read_states};
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
    ReadSets,
    ReadDecisions,
    ReadLexActions
}

/// Augmented Transition Network
///
/// Basically NFA(graph) of states and possible(maybe multiple) transitions on a given particular symbol.
///
#[derive(Clone)]
pub struct ATN {
    grammar_type: ATNType,

    // Keep positions while simulating
    heads: HashSet<ATNStateRef>,

    // max_token_type: usize,
    lexer_actions: Vec<LexerAction>,

    pub(super) states: Vec<ATNState>,
    pub(super) rules: Vec<ATNRule>,
    pub(super) rule_stack: VecDeque<ATNRuleRef>,
    pub(super) modes: Vec<ATNStateRef>,
    pub(super) sets: Vec<HashSet<usize>>,
    pub(super) transitions: Vec<Transition>,
    pub(super) decisions: Vec<ATNStateRef>,
    pub(super) lex_actions: Vec<LexerAction>
}

impl Debug for ATN {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ATN")
            .field("grammar_type", &self.grammar_type)
            // .field("max_token_type", &self.max_token_type)
            
            .field("states", &self.states)
            .field("rules", &self.rules)
            .field("modes", &self.modes)
            .field("sets", &self.sets)
            .field("transitions", &self.transitions)
            .field("decisions", &self.decisions)
            .field("lex_actions", &self.lex_actions)

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

        let mut states = read_states(&mut data).ok_or(ATNConstructionErr::ReadStates)?;
        
        let mut rules = read_rules(&mut data, grammar_type == ATNType::LEXER).ok_or(ATNConstructionErr::ReadRules)?;
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
        
        let transitions = read_edges(&mut data).ok_or(ATNConstructionErr::ReadEdges)?;
        for (index, transition) in transitions.iter().enumerate() {
            if let Some(state) = states.get_mut(transition.source()) {
                state.add_transition(index);
            }
        }

        let decisions = read_decisions(&mut data).ok_or(ATNConstructionErr::ReadDecisions)?;
        
        let lex_actions = if grammar_type == ATNType::LEXER {
            read_lex_actions(&mut data).ok_or(ATNConstructionErr::ReadLexActions)?
        } else {
            Vec::new()
        };

        Ok(ATN {
            grammar_type,
            heads: HashSet::new(),

            lexer_actions: Vec::new(),

            states,
            rules,
            rule_stack: VecDeque::new(),
            modes,
            sets,
            transitions,
            decisions,
            lex_actions,
        })
    }

    pub fn atn_type(&self) -> ATNType {
        self.grammar_type
    }

    pub(crate) fn set_head(&mut self, head: usize) {
        self.heads = HashSet::new();
        self.heads.insert(head);
    }
    
    pub(crate) fn set_heads(&mut self, heads: HashSet<ATNStateRef>) {
        self.heads = heads;
    }

    pub(crate) fn enter_rule(&mut self, rule: ATNRuleRef) {
        self.set_head(rule);
        self.rule_stack.push_back(rule);
    }

    pub(crate) fn exit_rule(&mut self) -> Option<ATNRuleRef> {
        self.rule_stack.pop_back()
    }
}