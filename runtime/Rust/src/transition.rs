use std::borrow::Cow;

use crate::atn_state::ATNStateRef;
use crate::interval_set::IntervalSet;
use crate::lexer::{LEXER_MAX_CHAR_VALUE, LEXER_MIN_CHAR_VALUE};
use crate::semantic_context::SemanticContext;

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq)]
pub enum TransitionType {
    TRANSITION_EPSILON = 1,
    TRANSITION_RANGE,
    TRANSITION_RULE,
    TRANSITION_PREDICATE,
    TRANSITION_ATOM,
    TRANSITION_ACTION,
    TRANSITION_SET,
    TRANSITION_NOTSET,
    TRANSITION_WILDCARD,
    TRANSITION_PRECEDENCE,
}

#[derive(Debug)]
pub enum Transition {
    Atom {
        target: ATNStateRef,
        label: i32,
    },
    Rule {
        target: ATNStateRef,
        follow_state: ATNStateRef,
        rule_index: i32,
        precedence: i32,
    },
    Epsilon {
        target: ATNStateRef,
        outermost_precedence_return: i32,
    },
    Range {
        target: ATNStateRef,
        start: i32,
        stop: i32,
    },
    Action {
        target: ATNStateRef,
        is_ctx_dependent: bool,
        rule_index: i32,
        action_index: i32,
        pred_index: i32,
    },
    Set {
        target: ATNStateRef,
        set: IntervalSet,
    },
    NotSet {
        target: ATNStateRef,
        set: IntervalSet,
    },
    Wildcard {
        target: ATNStateRef,
    },
    Predicate {
        target: ATNStateRef,
        is_ctx_dependent: bool,
        rule_index: i32,
        pred_index: i32,
    },
    PrecedencePredicate {
        target: ATNStateRef,
        precedence: i32,
    },
}

impl Transition {
    pub fn get_target(&self) -> ATNStateRef {
        match self {
            Self::Atom { target, .. }
            | Self::Rule { target, .. }
            | Self::Epsilon { target, .. }
            | Self::Range { target, .. }
            | Self::Action { target, .. }
            | Self::Set { target, .. }
            | Self::NotSet { target, .. }
            | Self::Wildcard { target }
            | Self::Predicate { target, .. }
            | Self::PrecedencePredicate { target, .. } => *target,
        }
    }

    pub fn set_target(&mut self, s: ATNStateRef) {
        match self {
            Self::Atom { target, .. }
            | Self::Rule { target, .. }
            | Self::Epsilon { target, .. }
            | Self::Range { target, .. }
            | Self::Action { target, .. }
            | Self::Set { target, .. }
            | Self::NotSet { target, .. }
            | Self::Wildcard { target }
            | Self::Predicate { target, .. }
            | Self::PrecedencePredicate { target, .. } => *target = s,
        }
    }

    pub fn is_epsilon(&self) -> bool {
        matches!(
            self,
            Self::Rule { .. }
            | Self::Epsilon { .. }
            | Self::Action { .. }
            | Self::Predicate { .. }
            | Self::PrecedencePredicate { .. }
        )
    }

    pub fn get_label(&self) -> Option<IntervalSet> {
        match self {
            Self::Atom { label, .. } => {
                let mut r = IntervalSet::new();
                r.add_one(*label);
                
                Some(r)
            }
            Self::Range { start, stop, .. } => {
                let mut r = IntervalSet::new();
                r.add_range(*start, *stop);

                Some(r)
            }
            Self::Set { set, .. } | Self::NotSet { set, .. } => Some(set.clone()),
            _ => None,
        }
    }

    // pub fn get_serialization_type(&self) -> i32 {
    //     match self {
    //         Self::Atom { .. } => TRANSITION_ATOM,
    //         Self::Rule { .. } => TRANSITION_RULE,
    //         Self::Epsilon { .. } => TRANSITION_EPSILON,
    //         Self::Range { .. } => TRANSITION_RANGE,
    //         Self::Action { .. } => TRANSITION_ACTION,
    //         Self::Set { .. } => TRANSITION_SET,
    //         Self::NotSet { .. } => TRANSITION_NOTSET,
    //         Self::Wildcard { .. } => TRANSITION_WILDCARD,
    //         Self::Predicate { .. } => TRANSITION_PREDICATE,
    //         Self::PrecedencePredicate { .. } => TRANSITION_PRECEDENCE,
    //     }
    // }

    pub fn matches(&self, symbol: i32, min_vocab_symbol: i32, max_vocab_symbol: i32) -> bool {
        match self {
            Self::Atom { label, .. } => symbol == *label,
            Self::Rule { .. } => unimplemented!(),
            Self::Epsilon { .. } | Self::Action { .. } | Self::Predicate { .. } | Self::PrecedencePredicate { .. } => false,
            Self::Range { start, stop, .. } => symbol >= *start && symbol <= *stop,
            Self::Set { set, .. } => set.contains(symbol),
            Self::NotSet { set, .. } => {
                symbol >= min_vocab_symbol && symbol <= max_vocab_symbol && !set.contains(symbol)
            }
            Self::Wildcard { .. } => symbol < max_vocab_symbol && symbol > min_vocab_symbol,
        }
    }

    pub fn get_predicate(&self) -> Option<SemanticContext> {
        match self {
            Self::Predicate {
                rule_index,
                pred_index,
                is_ctx_dependent,
                ..
            } => Some(SemanticContext::Predicate {
                rule_index: *rule_index,
                pred_index: *pred_index,
                is_ctx_dependent: *is_ctx_dependent,
            }),
            Self::PrecedencePredicate { precedence, .. } => {
                Some(SemanticContext::Precedence(*precedence))
            }
            _ => None,
        }
    }

    pub fn get_reachable_target(&self, symbol: i32) -> Option<ATNStateRef> {
        if self.matches(symbol, LEXER_MIN_CHAR_VALUE, LEXER_MAX_CHAR_VALUE) {
            return Some(self.get_target());
        }
        None
    }

}