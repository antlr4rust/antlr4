use std::fmt::Debug;
use std::slice::Iter;

use once_cell::sync::OnceCell;

use crate::interval_set::IntervalSet;
use crate::transition::Transition;

pub(crate) const ATNSTATE_INVALID_TYPE: i32 = 0;
pub(crate) const ATNSTATE_BASIC: i32 = 1;
pub(crate) const ATNSTATE_RULE_START: i32 = 2;
pub(crate) const ATNSTATE_BLOCK_START: i32 = 3;
pub(crate) const ATNSTATE_PLUS_BLOCK_START: i32 = 4;
pub(crate) const ATNSTATE_STAR_BLOCK_START: i32 = 5;
pub(crate) const ATNSTATE_TOKEN_START: i32 = 6;
pub(crate) const ATNSTATE_RULE_STOP: i32 = 7;
pub(crate) const ATNSTATE_BLOCK_END: i32 = 8;
pub(crate) const ATNSTATE_STAR_LOOP_BACK: i32 = 9;
pub(crate) const ATNSTATE_STAR_LOOP_ENTRY: i32 = 10;
pub(crate) const ATNSTATE_PLUS_LOOP_BACK: i32 = 11;
pub(crate) const ATNSTATE_LOOP_END: i32 = 12;
pub(crate) const ATNSTATE_INVALID_STATE_NUMBER: i32 = -1;

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
        decision: i32,
        nongreedy: bool,
        state: ATNDecisionState,
    },
}

impl ATNStateType {
    pub fn new(type_index: i32, next: Option<i32>) -> Result<Self, ()> {

        match type_index {
            ATNSTATE_BASIC => Ok(ATNStateType::BasicState),
            ATNSTATE_RULE_START => Ok(ATNStateType::RuleStartState {
                stop_state: 0,
                is_left_recursive: false,
            }),
            ATNSTATE_BLOCK_START => Ok(ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::BlockStartState {
                    end_state: next?,
                    en: ATNBlockStart::BasicBlockStart,
                },
            }),
            ATNSTATE_PLUS_BLOCK_START => Ok(ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::BlockStartState {
                    end_state: next?,
                    en: ATNBlockStart::PlusBlockStart(0),
                },
            }),
            ATNSTATE_STAR_BLOCK_START => Ok(ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::BlockStartState {
                    end_state: next?,
                    en: ATNBlockStart::StarBlockStart,
                },
            }),
            ATNSTATE_TOKEN_START => Ok(ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::TokenStartState,
            }),
            ATNSTATE_RULE_STOP => Ok(ATNStateType::RuleStopState),
            ATNSTATE_BLOCK_END => Ok(ATNStateType::BlockEndState(0)),
            ATNSTATE_STAR_LOOP_BACK => Ok(ATNStateType::StarLoopbackState),
            ATNSTATE_STAR_LOOP_ENTRY => Ok(ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::StarLoopEntry {
                    loop_back_state: 0,
                    is_precedence: false,
                },
            }),
            ATNSTATE_PLUS_LOOP_BACK => Ok(ATNStateType::DecisionState {
                decision: -1,
                nongreedy: false,
                state: ATNDecisionState::PlusLoopBack,
            }),
            ATNSTATE_LOOP_END => Ok(ATNStateType::LoopEndState(next?)),

            _ => Err(()),
        }
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

pub type ATNStateRef = i32;

#[derive(Debug)]
pub struct ATNState {
    next_tokens_within_rule: OnceCell<IntervalSet>,

    epsilon_only_transitions: bool,

    pub rule_index: i32,

    pub state_number: i32,

    pub state_type: ATNStateType,

    transitions: Vec<Box<Transition>>,
}

impl ATNState {
    pub fn new(state_type: ATNStateType, rule_index: i32, state_number: i32 ) -> Self {
        Self {
            next_tokens_within_rule: OnceCell::new(),
            epsilon_only_transitions: false,
            rule_index,
            state_number,
            state_type,
            transitions: Vec::new(),
        }
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
    fn get_rule_index(&self) -> i32 {
        self.rule_index
    }

    fn set_rule_index(&self, _v: i32) {
        unimplemented!()
    }

    fn get_next_tokens_within_rule(&self) -> &OnceCell<IntervalSet> {
        &self.next_tokens_within_rule
    }

    pub fn get_state_type(&self) -> &ATNStateType {
        &self.state_type
    }

    pub fn get_state_type_mut(&mut self) -> &mut ATNStateType {
        &mut self.state_type
    }

    fn get_state_number(&self) -> i32 {
        self.state_number
    }

    fn set_state_number(&self, _state_number: i32) {
        unimplemented!()
    }

    fn get_transitions(&self) -> &Vec<Box<Transition>> {
        &self.transitions
    }

    fn set_transitions(&self, _t: Vec<Box<Transition>>) {
        unimplemented!()
    }

    fn add_transition(&mut self, trans: Box<Transition>) {
        if self.transitions.is_empty() {
            self.epsilon_only_transitions = trans.is_epsilon()
        } else {
            self.epsilon_only_transitions &= trans.is_epsilon()
        }

        let mut already_present = false;
        for existing in self.transitions.iter() {
            if existing.get_target() == trans.get_target() {
                if existing.get_label().is_some()
                    && trans.get_label().is_some()
                    && existing.get_label() == trans.get_label()
                {
                    already_present = true;
                    break;
                } else if existing.is_epsilon() && trans.is_epsilon() {
                    already_present = true;
                    break;
                }
            }
        }
        if !already_present {
            self.transitions.push(trans);
        }
    }
}
