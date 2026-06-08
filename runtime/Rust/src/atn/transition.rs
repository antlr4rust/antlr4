use std::slice::Iter;

use crate::atn::ATNStateRef;

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq)]
pub enum TransitionType {
    TRANSITION_EPSILON,
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

impl TransitionType {
    pub fn new(t: usize) -> Option<Self> {
        let t = match t {
            1 => TransitionType::TRANSITION_EPSILON,
            2 => TransitionType::TRANSITION_RANGE,
            3 => TransitionType::TRANSITION_RULE,
            4 => TransitionType::TRANSITION_PREDICATE,
            5 => TransitionType::TRANSITION_ATOM,
            6 => TransitionType::TRANSITION_ACTION,
            7 => TransitionType::TRANSITION_SET,
            8 => TransitionType::TRANSITION_NOTSET,
            9 => TransitionType::TRANSITION_WILDCARD,
            10 => TransitionType::TRANSITION_PRECEDENCE,
            _ => return None
        };

        Some(t)
    }
}

#[derive(Debug)]
pub enum Transition {
    // Epsilon transitions
    Rule {
        source: ATNStateRef,
        target: ATNStateRef,
        follow_state: ATNStateRef,
        rule_index: usize,
        precedence: usize,
    },
    Epsilon {
        source: ATNStateRef,
        target: ATNStateRef,
        outermost_precedence_return: usize,
    },
    Action {
        source: ATNStateRef,
        target: ATNStateRef,
        is_ctx_dependent: bool,
        rule_index: usize,
        action_index: usize,
        pred_index: usize,
    },
    Predicate {
        source: ATNStateRef,
        target: ATNStateRef,
        is_ctx_dependent: bool,
        rule_index: usize,
        pred_index: usize,
    },
    PrecedencePredicate {
        source: ATNStateRef,
        target: ATNStateRef,
        precedence: usize,
    },

    // Non-epsilon transitions
    Atom {
        source: ATNStateRef,
        target: ATNStateRef,
        label: usize,
    },

    Range {
        source: ATNStateRef,
        target: ATNStateRef,
        start: usize,
        stop: usize,
    },
    // Set {
    //     target: ATNStateRef,
    //     set: IntervalSet,
    // },
    // NotSet {
    //     target: ATNStateRef,
    //     set: IntervalSet,
    // },
    Wildcard {
        source: ATNStateRef,
        target: ATNStateRef,
    },

}

impl Transition {
    pub fn new(data: &mut Iter<usize>) -> Option<Self> {
        let source: ATNStateRef = *data.next()?;
        let target: ATNStateRef = *data.next()?;
        let transition_type = TransitionType::new(*data.next()?)?;

        let (arg1, arg2, arg3) = (*data.next()?, *data.next()?, *data.next()?);
        
        let t = match transition_type {
            TransitionType::TRANSITION_EPSILON => Self::Epsilon { source, target, outermost_precedence_return: 0 },
            _ => { return None }
        };


        Some(t)
    }
    
    pub fn source(&self) -> ATNStateRef {
        match self {
            Self::Atom { source, .. }
            | Self::Rule { source, .. }
            | Self::Epsilon { source, .. }
            | Self::Range { source, .. }
            | Self::Action { source, .. }
            // | Self::Set { source, .. }
            // | Self::NotSet { source, .. }
            | Self::Wildcard { source, .. }
            | Self::Predicate { source, .. }
            | Self::PrecedencePredicate { source, .. } => *source,
        }
    }

    pub fn target(&self) -> ATNStateRef {
        match self {
            Self::Atom { target, .. }
            | Self::Rule { target, .. }
            | Self::Epsilon { target, .. }
            | Self::Range { target, .. }
            | Self::Action { target, .. }
            // | Self::Set { target, .. }
            // | Self::NotSet { target, .. }
            | Self::Wildcard { target, .. }
            | Self::Predicate { target, .. }
            | Self::PrecedencePredicate { target, .. } => *target,
        }
    }

    // pub fn set_target(&mut self, s: ATNStateRef) {
    //     match self {
    //         Self::Atom { target, .. }
    //         | Self::Rule { target, .. }
    //         | Self::Epsilon { target, .. }
    //         | Self::Range { target, .. }
    //         | Self::Action { target, .. }
    //         | Self::Set { target, .. }
    //         | Self::NotSet { target, .. }
    //         | Self::Wildcard { target }
    //         | Self::Predicate { target, .. }
    //         | Self::PrecedencePredicate { target, .. } => *target = s,
    //     }
    // }

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
}

    // pub fn get_label(&self) -> Option<IntervalSet> {
    //     match self {
    //         Self::Atom { label, .. } => {
    //             let mut r = IntervalSet::new();
    //             r.add_one(*label);
                
    //             Some(r)
    //         }
    //         Self::Range { start, stop, .. } => {
    //             let mut r = IntervalSet::new();
    //             r.add_range(*start, *stop);

    //             Some(r)
    //         }
    //         Self::Set { set, .. } | Self::NotSet { set, .. } => Some(set.clone()),
    //         _ => None,
    //     }
    // }

    // pub fn matches(&self, symbol: usize, min_vocab_symbol: usize, max_vocab_symbol: usize) -> bool {
    //     match self {
    //         Self::Atom { label, .. } => symbol == *label,
    //         Self::Rule { .. } => unimplemented!(),
    //         Self::Epsilon { .. } | Self::Action { .. } | Self::Predicate { .. } | Self::PrecedencePredicate { .. } => false,
    //         Self::Range { start, stop, .. } => symbol >= *start && symbol <= *stop,
    //         Self::Set { set, .. } => set.contains(symbol),
    //         Self::NotSet { set, .. } => {
    //             symbol >= min_vocab_symbol && symbol <= max_vocab_symbol && !set.contains(symbol)
    //         }
    //         Self::Wildcard { .. } => symbol < max_vocab_symbol && symbol > min_vocab_symbol,
    //     }
    // }

    // pub fn get_predicate(&self) -> Option<SemanticContext> {
    //     match self {
    //         Self::Predicate {
    //             rule_index,
    //             pred_index,
    //             is_ctx_dependent,
    //             ..
    //         } => Some(SemanticContext::Predicate {
    //             rule_index: *rule_index,
    //             pred_index: *pred_index,
    //             is_ctx_dependent: *is_ctx_dependent,
    //         }),
    //         Self::PrecedencePredicate { precedence, .. } => {
    //             Some(SemanticContext::Precedence(*precedence))
    //         }
    //         _ => None,
    //     }
    // }

    // pub fn get_reachable_target(&self, symbol: usize) -> Option<ATNStateRef> {
    //     if self.matches(symbol, LEXER_MIN_CHAR_VALUE, LEXER_MAX_CHAR_VALUE) {
    //         return Some(self.get_target());
    //     }
    //     None
    // }
