use std::{collections::HashSet, range::Range};
use std::fmt::Debug;

use crate::atn::{ATNStateRef, ATNTransitionRef};

pub(crate) const ATNSTATE_BASIC: usize = 1;
pub(crate) const ATNSTATE_RULE_START: usize = 2;
pub(crate) const ATNSTATE_BLOCK_START: usize = 3;
pub(crate) const ATNSTATE_PLUS_BLOCK_START: usize = 4;
pub(crate) const ATNSTATE_STAR_BLOCK_START: usize = 5;
pub(crate) const ATNSTATE_TOKEN_START: usize = 6;
pub(crate) const ATNSTATE_RULE_STOP: usize = 7;
pub(crate) const ATNSTATE_BLOCK_END: usize = 8;
pub(crate) const ATNSTATE_STAR_LOOP_BACK: usize = 9;
pub(crate) const ATNSTATE_STAR_LOOP_ENTRY: usize = 10;
pub(crate) const ATNSTATE_PLUS_LOOP_BACK: usize = 11;
pub(crate) const ATNSTATE_LOOP_END: usize = 12;

//might be changed later
#[doc(hidden)]
#[derive(Debug, Eq, PartialEq)]
pub enum ATNStateType {
    RuleStartState {
        stop_state: ATNStateRef,
        is_left_recursive: bool,
    },
    RuleStopState,
    BlockEndState(ATNStateRef),
    LoopEndState(ATNStateRef),
    StarLoopbackState,
    BasicState,
    DecisionState {
        decision: usize,
        nongreedy: bool,
        state: ATNDecisionState,
    },
}

impl ATNStateType {
    pub fn new(state_type: usize, rule_index: usize) -> Result<Self, ()> {
        Ok(
            match state_type {
                ATNSTATE_BASIC => ATNStateType::BasicState,
                ATNSTATE_RULE_START => ATNStateType::RuleStartState {
                    stop_state: 0,
                    is_left_recursive: false,
                },
                ATNSTATE_BLOCK_START => ATNStateType::DecisionState {
                    decision: usize::MAX,
                    nongreedy: false,
                    state: ATNDecisionState::BlockStartState {
                        end_state: 0,
                        en: ATNBlockStart::BasicBlockStart,
                    },
                },
                ATNSTATE_PLUS_BLOCK_START => ATNStateType::DecisionState {
                    decision: usize::MAX,
                    nongreedy: false,
                    state: ATNDecisionState::BlockStartState {
                        end_state: 0,
                        en: ATNBlockStart::PlusBlockStart(0),
                    },
                },
                ATNSTATE_STAR_BLOCK_START => ATNStateType::DecisionState {
                    decision: usize::MAX,
                    nongreedy: false,
                    state: ATNDecisionState::BlockStartState {
                        end_state: 0,
                        en: ATNBlockStart::StarBlockStart,
                    },
                },
                ATNSTATE_TOKEN_START => ATNStateType::DecisionState {
                    decision: usize::MAX,
                    nongreedy: false,
                    state: ATNDecisionState::TokenStartState,
                },
                ATNSTATE_RULE_STOP => ATNStateType::RuleStopState,
                ATNSTATE_BLOCK_END => ATNStateType::BlockEndState(0),
                ATNSTATE_STAR_LOOP_BACK => ATNStateType::StarLoopbackState,
                ATNSTATE_STAR_LOOP_ENTRY => ATNStateType::DecisionState {
                    decision: usize::MAX,
                    nongreedy: false,
                    state: ATNDecisionState::StarLoopEntry {
                        loop_back_state: 0,
                        is_precedence: false,
                    },
                },
                ATNSTATE_PLUS_LOOP_BACK => ATNStateType::DecisionState {
                    decision: usize::MAX,
                    nongreedy: false,
                    state: ATNDecisionState::PlusLoopBack,
                },
                ATNSTATE_LOOP_END => ATNStateType::LoopEndState(0),
                _ => {
                    return Err(())
                }
            }
        )
    }
}

#[doc(hidden)]
#[derive(Debug, Eq, PartialEq)]
pub enum ATNDecisionState {
    StarLoopEntry {
        loop_back_state: ATNStateRef,
        is_precedence: bool,
    },
    TokenStartState,
    PlusLoopBack,
    BlockStartState {
        end_state: ATNStateRef,
        en: ATNBlockStart,
    },
}

#[doc(hidden)]
#[derive(Debug, Eq, PartialEq)]
pub enum ATNBlockStart {
    BasicBlockStart,
    StarBlockStart,
    PlusBlockStart(ATNStateRef),
}

#[derive(Debug)]
pub struct ATNState {
    next_tokens_within_rule: HashSet<Range<usize>>,

    epsilon_only_transitions: bool,

    pub rule_index: usize,

    pub state_number: usize,

    pub state_type: ATNStateType,

    transitions: HashSet<ATNTransitionRef>,
}

impl ATNState {
    pub fn new(state_type: usize, rule_index: usize, state_number: usize ) -> Option<Self> {
        Some(Self {
            next_tokens_within_rule: HashSet::new(),
            epsilon_only_transitions: false,
            rule_index,
            state_number,
            state_type: ATNStateType::new(state_type, rule_index).ok()?,
            transitions: HashSet::new(),
        })
    }

    pub fn set_nongreedy(&mut self, ng: bool) {
        if let ATNStateType::DecisionState {
            mut nongreedy,
            ..
        } = self.state_type {
            nongreedy = ng;
        }
    }

    pub fn set_left_recursive(&mut self, lr: bool) {
        if let ATNStateType::RuleStartState {
            mut is_left_recursive,
            ..
        } = self.state_type {
            is_left_recursive = lr;
        }
    }

    fn has_epsilon_only_transitions(&self) -> bool {
        self.epsilon_only_transitions
    }
    pub fn get_rule_index(&self) -> usize {
        self.rule_index
    }

    fn set_rule_index(&self, _v: usize) {
        unimplemented!()
    }

    fn get_next_tokens_within_rule(&self) -> &HashSet<Range<usize>> {
        &self.next_tokens_within_rule
    }

    pub fn get_state_type(&self) -> &ATNStateType {
        &self.state_type
    }

    pub fn get_state_type_mut(&mut self) -> &mut ATNStateType {
        &mut self.state_type
    }

    pub fn get_state_number(&self) -> usize {
        self.state_number
    }

    fn set_state_number(&self, _state_number: usize) {
        unimplemented!()
    }

    pub fn transitions(&self) -> &HashSet<ATNTransitionRef> {
        &self.transitions
    }

    fn add_transition(&mut self, t: ATNTransitionRef) {
        self.transitions.insert(t);
    }
}
