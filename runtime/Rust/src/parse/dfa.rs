use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::atn::ATN;
use crate::atn_state::{ATNDecisionState, ATNStateRef, ATNStateType};
use crate::dfa_state::{DFAState, DFAStateRef};
use crate::lexer_atn_simulator::ERROR_DFA_STATE_REF;
use crate::vocabulary::Vocabulary;

#[derive(Eq, PartialEq, Debug)]
pub struct PredPrediction {
    pub(crate) alt: i32,
    pub(crate) pred: SemanticContext,
}

impl Display for PredPrediction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!("({},{:?})", self.alt, self.pred))
    }
}

//index in DFA.states
pub type DFAStateRef = usize;

#[derive(Eq, Debug)]
pub struct DFAState {
    /// Number of this state in corresponding DFA
    pub state_number: usize,
    pub configs: Box<ATNConfigSet>,
    /// - 0 => no edge
    /// - usize::MAX => error edge
    /// - _ => actual edge
    pub edges: Vec<DFAStateRef>,
    pub is_accept_state: bool,

    pub prediction: i32,
    pub(crate) lexer_action_executor: Option<Box<LexerActionExecutor>>,
    pub requires_full_context: bool,
    pub predicates: Vec<PredPrediction>,
}

impl Display for DFAState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut base_str = format!(
            "{}s{}{}",
            if self.is_accept_state { ":" } else { "" },
            self.state_number - 1,
            if self.requires_full_context { "^" } else { "" },
        );
        if self.is_accept_state {
            base_str = if !self.predicates.is_empty() {
                unimplemented!()
            //                format!("{}=>{:?}", base_str, state.predicates)
            } else {
                format!("{}=>{}", base_str, self.prediction)
            };
        }

        f.write_str(&base_str)
    }
}

impl PartialEq for DFAState {
    fn eq(&self, other: &Self) -> bool {
        self.configs == other.configs
    }
}

impl Hash for DFAState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.configs.hash(state);
    }
}

impl DFAState {
    pub fn default_hash(&self) -> u64 {
        let mut hasher = MurmurHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn new_dfastate(state_number: usize, configs: Box<ATNConfigSet>) -> DFAState {
        DFAState {
            state_number,
            configs,
            //            edges: Vec::with_capacity((MAX_DFA_EDGE - MIN_DFA_EDGE + 1) as usize),
            edges: Vec::new(),
            is_accept_state: false,
            prediction: 0,
            lexer_action_executor: None,
            requires_full_context: false,
            predicates: Vec::new(),
        }
    }

    //    fn get_alt_set(&self) -> &Set { unimplemented!() }

    // fn set_prediction(&self, _v: i32) { unimplemented!() }
}

#[derive(Debug)]
pub struct DFA {
    decision: i32,

    /// Set of all dfa states.
    states: HashSet<DFAState>,

    /// Initial DFA state
    s0: Option<DFAStateRef>,

    is_precedence_dfa: bool,
}

impl DFA {
    pub fn new(atn: Arc<ATN>, atn_start_state: ATNStateRef, decision: i32) -> DFA {
        let mut dfa = DFA {
            decision,
            states: HashSet::new(),
            s0: None,
            is_precedence_dfa: false,
        };

        // to indicate null
        dfa.states.push(DFAState::new_dfastate(
            usize::max_value(),
            Box::new(ATNConfigSet::new_base_atnconfig_set(true)),
        ));
        if let ATNStateType::DecisionState {
            state:
                ATNDecisionState::StarLoopEntry {
                    is_precedence: true,
                    ..
                },
            ..
        } = atn.states[atn_start_state as usize].get_state_type()
        {
            dfa.is_precedence_dfa = true;
            let mut precedence_state = DFAState::new_dfastate(
                dfa.states.len(),
                Box::new(ATNConfigSet::new_base_atnconfig_set(true)),
            );
            precedence_state.edges = vec![];
            precedence_state.is_accept_state = false;
            precedence_state.requires_full_context = false;

            dfa.s0 = Some(precedence_state.state_number);
            dfa.states.push(precedence_state)
        }
        dfa
    }

    pub fn get_precedence_start_state(&self, _precedence: i32) -> Option<DFAStateRef> {
        if !self.is_precedence_dfa {
            panic!("dfa is supposed to be precedence here");
        }

        self.s0.and_then(|s0| {
            self.states[s0]
                .edges
                .get(_precedence as usize)
                .and_then(|it| match *it {
                    0 => None,
                    x => Some(x),
                })
        })
    }

    pub fn set_precedence_start_state(&mut self, precedence: i32, _start_state: DFAStateRef) {
        if !self.is_precedence_dfa {
            panic!("set_precedence_start_state called for not precedence dfa")
        }

        if precedence < 0 {
            return;
        }
        let precedence = precedence as usize;

        if let Some(x) = &self.s0 {
            self.states[*x].edges.apply(|edges| {
                if edges.len() <= precedence {
                    edges.resize(precedence + 1, 0);
                }
                edges[precedence] = _start_state;
            });
        }
    }

    pub fn is_precedence_dfa(&self) -> bool {
        self.is_precedence_dfa
    }

    pub fn set_precedence_dfa(&mut self, precedence_dfa: bool) {
        self.is_precedence_dfa = precedence_dfa
    }

    pub fn to_string(&self, vocabulary: &Vocabulary) -> String {
        let mut string = String::new();

        if self.s0.is_none() {
            return string;
        }

        for source in self.states.iter() {
            for (i, edge) in source.edges.iter().copied().enumerate() {
                if edge != 0 && edge != ERROR_DFA_STATE_REF {
                    let target = &self.states[edge];
                    write!(
                        string,
                        "{}-{}->{}\n",
                        source,
                        vocabulary.get_display_name(i as i32 - 1),
                        target
                    )?;
                }
            }
        }

        string
    }

    pub fn to_lexer_string(&self) -> String {
        let mut string = String::new();

        if self.s0.is_none() {
            return string;
        }

        for source in self.states.iter() {
            for (i, edge) in source.edges.iter().copied().enumerate() {
                if edge != 0 && edge != ERROR_DFA_STATE_REF {
                    let target = &self.states[edge];
                    write!(
                        string,
                        "{}-{}->{}\n",
                        source,
                        char::try_from(i as u32),
                        target
                    )?;
                }
            }
        }

        string
    }
}