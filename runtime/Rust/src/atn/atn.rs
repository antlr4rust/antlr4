use std::collections::HashMap;
use std::io::Cursor;
use std::slice::Iter;

use crate::atn_state::ATNState;
use crate::atn_state::ATNStateRef;
use crate::atn_state::ATNStateType;
use crate::atn_type::ATNType;
use crate::dfa::ScopeExt;
use crate::interval_set::IntervalSet;
use crate::lexer_action::LexerAction;
use crate::ll1_analyzer::LL1Analyzer;
use crate::parser::ParserNodeType;
use crate::rule_context::EmptyContextType;
use crate::token::{TOKEN_EOF, TOKEN_EPSILON};
use crate::token_factory::CommonTokenFactory;
use crate::transition::Transition;
use std::fmt::{Debug, Formatter};
#[doc(hidden)]
#[derive(Eq, PartialEq, Debug)]
pub enum ATNType {
    LEXER = 0,
    PARSER,
}

impl ATNType {
    pub fn new(i: i32) -> Result<ATNType, ()> {
        match i {
            0 => Ok(Self::LEXER),
            1 => Ok(Self::PARSER),
            _ => Err(()),
        }
    }
}

pub const INVALID_ALT: i32 = 0;

enum ATNConstructionErr {
    VersionMismatch,
}
/// Augmented Transition Network
///
/// Basically NFA(graph) of states and possible(maybe multiple) transitions on a given particular symbol.
///
pub struct ATN {
    pub decision_states: Vec<ATNStateRef>,

    pub grammar_type: ATNType,

    pub(crate) lexer_actions: Vec<LexerAction>,

    pub max_token_type: i32,

    pub mode_name_to_start_state: HashMap<String, ATNStateRef>,

    pub mode_to_start_state: Vec<ATNStateRef>,

    pub rule_to_start_state: Vec<ATNStateRef>,

    pub rule_to_stop_state: Vec<ATNStateRef>,

    pub rule_to_token_type: Vec<i32>,

    pub states: Vec<ATNState>,
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
    const SERIALIZED_VERSION: i32 = 4;

    pub fn from_serialized(data: Iter<i32>, verify_atn: bool) -> Result<Self, ATNConstructionErr> {
        if data.next()? != Self::SERIALIZED_VERSION {
            return Err(ATNConstructionErr::VersionMismatch);
        };

        let grammar_type = ATNType::new(*data.next()?)?;
        let max_token_type = *data.next()?;

        let in_construction_atn = ATN {
            decision_states: Vec::new(),
            grammar_type,
            lexer_actions: Vec::new(),
            max_token_type,
            mode_name_to_start_state: HashMap::new(),
            mode_to_start_state: Vec::new(),
            rule_to_start_state: Vec::new(),
            rule_to_stop_state: Vec::new(),
            rule_to_token_type: Vec::new(),
            states: Vec::new(),
        };
    }

    ///Compute the set of valid tokens that can occur starting in `s` and
    ///staying in same rule. `Token::EPSILON` is in set if we reach end of
    ///rule.
    pub fn next_tokens<'a>(&self, s: &ATNState) -> &'a IntervalSet {
        s.get_next_tokens_within_rule().get_or_init(|| {
            self.next_tokens_in_ctx::<EmptyContextType<'_, CommonTokenFactory>>(s, None)
                .modify_with(|r| r.read_only = true)
        })
    }

    /// Compute the set of valid tokens that can occur starting in state `s`.
    /// If `ctx` is null, the set of tokens will not include what can follow
    /// the rule surrounding `s`. In other words, the set will be
    /// restricted to tokens reachable staying within `s`'s rule.
    pub fn next_tokens_in_ctx<'a, Ctx: ParserNodeType<'a>>(
        &self,
        s: &ATNState,
        _ctx: Option<&Ctx::Type>,
    ) -> IntervalSet {
        let analyzer = LL1Analyzer::new(self);
        analyzer.look::<Ctx>(s, None, _ctx)
    }

    pub(crate) fn add_state(&mut self, state: ATNState) {
        debug_assert_eq!(state.get_state_number() as usize, self.states.len());
        self.states.push(state)
    }

    // fn remove_state(&self, _state: ATNStateRef) { unimplemented!() }

    // fn define_decision_state(&self, _s: ATNStateRef) -> i32 { unimplemented!() }

    pub fn get_decision_states(&self) -> &Vec<ATNStateRef> {
        &self.decision_states
    }

    /// Computes the set of input symbols which could follow ATN state number
    /// {@code stateNumber} in the specified full {@code context}. This method
    /// considers the complete parser context, but does not evaluate semantic
    /// predicates (i.e. all predicates encountered during the calculation are
    /// assumed true). If a path in the ATN exists from the starting state to the
    /// {@link RuleStopState} of the outermost context without matching any
    /// symbols, {@link Token#EOF} is added to the returned set.
    ///
    /// <p>If {@code context} is {@code null}, it is treated as {@link ParserRuleContext#EMPTY}.</p>
    ///
    /// Note that this does NOT give you the set of all tokens that could
    /// appear at a given token position in the input phrase.  In other words,
    /// it does not answer:
    ///
    ///   "Given a specific partial input phrase, return the set of all tokens
    ///    that can follow the last token in the input phrase."
    ///
    /// The big difference is that with just the input, the parser could
    /// land right in the middle of a lookahead decision. Getting
    /// all *possible* tokens given a partial input stream is a separate
    /// computation. See https://github.com/antlr/antlr4/issues/1428
    ///
    /// For this function, we are specifying an ATN state and call stack to compute
    /// what token(s) can come next and specifically: outside of a lookahead decision.
    /// That is what you want for error reporting and recovery upon parse error.
    ///
    /// @param stateNumber the ATN state number
    /// @param context the full parse context
    /// @return The set of potentially valid input symbols which could follow the
    /// specified state in the specified context.
    /// Panics if the ATN does not contain a state with
    /// number {@code stateNumber}
    pub fn get_expected_tokens(
        &self,
        state_number: i32,
        states_stack: impl Iterator<Item = i32>, // _ctx: &Rc<Ctx::Type>,
    ) -> IntervalSet {
        let s = self.states[state_number as usize].as_ref();
        let mut following = self.next_tokens(s);
        if !following.contains(TOKEN_EPSILON) {
            return following.clone();
        }
        let mut expected = IntervalSet::new();
        expected.add_set(following);
        expected.remove_one(TOKEN_EPSILON);
        // let mut ctx = Some(Rc::clone(_ctx));

        for state in states_stack {
            if !following.contains(TOKEN_EPSILON) {
                break;
            }

            let invoking_state = self.states[state as usize].as_ref();
            let tr = invoking_state.get_transitions().first().unwrap().as_ref();
            let tr = tr.cast::<RuleTransition>();
            following = self.next_tokens(self.states[tr.follow_state as usize].as_ref());
            expected.add_set(following);
            expected.remove_one(TOKEN_EPSILON);
            // ctx = c.get_parent_ctx();
        }

        if following.contains(TOKEN_EPSILON) {
            expected.add_one(TOKEN_EOF);
        }
        expected
    }
}


impl ATN {
        // self.read_rules(&mut atn, data);
        // self.read_modes(&mut atn, data);

        // let sets = self.read_sets(&mut atn, data);

        // self.read_edges(&mut atn, data, &sets);
        // self.read_decisions(&mut atn, data);
        // if atn.grammar_type == ATNType::LEXER {
        //     self.read_lexer_actions(&mut atn, data);
        // }
        // self.mark_precedence_decisions(&mut atn, data);
        // if self.deserialization_options.is_verify() {
        //     self.verify_atn(&mut atn, data);
        // }


    fn read_states(data: &mut Iter<i32>) -> Vec<ATNState> {
        let mut states = Vec::new();
        let state_count = *data.next()?;

        for i in 0..state_count {
            let type_index = *data.next()?;
            let rule_index = *data.next()?;

            let state = ATNState::new(ATNStateType::new(type_index, *data.next())?, rule_index, i);

            states.push(state);
        }

        let nongreedy_count = *data.next()?;
        for _ in 0..nongreedy_count {
            let state_index = *data.next()?;
            if let Some(s) = states.get(state_index as usize) {
                s.set_nongreedy(true);
            }
        }

        let left_recursive_count = *data.next()?;
        for _ in 0..nongreedy_count {
            let state_index = *data.next()?;
            if let Some(s) = states.get(state_index as usize) {
                s.set_left_recursive(true);
            }
        }

        states
    }

    fn read_rules(&self, atn: &mut ATN, data: &mut Iter<i32>) {
        let nrules = *data.next()? as usize;
        //        if atn.grammar_type == ATNType::LEXER {
        //            atn.rule_to_token_type.resize(nrules, 0)
        //        }

        atn.rule_to_start_state.resize(nrules, 0);
        for i in 0..nrules {
            let s = *data.next()?;
            atn.rule_to_start_state[i] = s;
            if atn.grammar_type == ATNType::LEXER {
                let token_type = *data.next()?;

                atn.rule_to_token_type.push(token_type);
            }
        }
        //println!("rule_to_token_type {:?}", atn.rule_to_token_type);
        //println!("rule_to_start_state {:?}", atn.rule_to_start_state);

        atn.rule_to_stop_state.resize(nrules, 0);
        for i in 0..atn.states.len() {
            let state = atn.states.get(i)?;
            if let ATNStateType::RuleStopState = state.get_state_type() {
                let rule_index = state.get_rule_index() as usize;
                atn.rule_to_stop_state[rule_index] = i as i32;
                let start_state = atn
                    .states
                    .get_mut(atn.rule_to_start_state[rule_index] as usize)
                    ?;
                if let ATNStateType::RuleStartState {
                    stop_state: stop, ..
                } = start_state.get_state_type_mut()
                {
                    *stop = i as i32
                }
            }
        }
    }

    fn read_modes(data: &mut Iter<i32>) -> Result<Vec<i32>, ()> {
        let mut mode_to_start_state = Vec::new();

        let nmodes = *data.next()?;
        for _ in 0..nmodes {
            mode_to_start_state.push(*data.next()?);
        }

        Ok(mode_to_start_state)
    }

    fn read_sets(data: &mut Iter<i32>) -> Result<Vec<IntervalSet>, ()> {
        let nsets = *data.next()?;
        let mut sets = Vec::new();

        for _ in 0..nsets {
            let intervals = *data.next()?;

            let mut set = IntervalSet::new();

            // check if contains eof
            if *data.next()? != 0 {
                set.add_one(-1)
            }

            for _ in 0..intervals {
                set.add_range(*data.next()?, *data.next()?);
            }
            sets.push(set);
        }

        sets
    }

    fn read_edges(&self, atn: &mut ATN, data: &mut Iter<i32>, sets: &Vec<IntervalSet>) {
        let nedges = *data.next()?;

        for _i in 0..nedges {
            let src = *data.next()?;
            let trg = *data.next()?;
            let ttype = *data.next()?;
            let arg1 = *data.next()?;
            let arg2 = *data.next()?;
            let arg3 = *data.next()?;

            let transition = self.edge_factory(atn, ttype, src, trg, arg1, arg2, arg3, sets);

            atn.states
                .get_mut(src as usize)
                ?
                .add_transition(transition);
        }

        let mut new_tr = Vec::new();
        for i in &atn.states {
            for tr in i.get_transitions() {
                match tr.get_serialization_type() {
                    TransitionType::TRANSITION_RULE => {
                        //                        println!("TRANSITION_RULE");
                        let tr = tr.as_ref().cast::<RuleTransition>();
                        let target = atn.states.get(tr.get_target() as usize)?;

                        let outermost_prec_return = if let ATNStateType::RuleStartState {
                            is_left_recursive: true,
                            ..
                        } = atn
                            .states
                            .get(atn.rule_to_start_state[target.get_rule_index() as usize] as usize)
                            ?
                            .get_state_type()
                        {
                            if tr.precedence == 0 {
                                target.get_rule_index() as i32
                            } else {
                                -1
                            }
                        } else {
                            -1
                        };

                        let return_tr = EpsilonTransition {
                            target: tr.follow_state,
                            outermost_precedence_return: outermost_prec_return,
                        };
                        new_tr.push((
                            atn.rule_to_stop_state[target.get_rule_index() as usize],
                            Box::new(return_tr),
                        ));
                    }
                    _ => continue,
                }
            }
        }
        new_tr
            .drain(..)
            .for_each(|(state, tr)| atn.states[state as usize].add_transition(tr));

        for i in 0..atn.states.len() {
            let atn_state = atn.states.get(i)?;
            match atn_state.get_state_type() {
                ATNStateType::DecisionState {
                    state:
                        ATNDecisionState::BlockStartState {
                            end_state: _,
                            en: _,
                        },
                    ..
                } => {

                }

                _x => { /*println!("{:?}",x);*/ }
            }
        }
    }

    fn read_decisions(&self, atn: &mut ATN, data: &mut Iter<i32>) {
        let ndecisions = *data.next()?;
        for i in 0..ndecisions {
            let s = *data.next()?;
            if let Some(dec_state) = atn.states.get_mut(s as usize) {
                atn.decision_states.push(s);

                if let ATNStateType::DecisionState { decision, .. } = dec_state.get_state_type_mut() {
                    *decision = i
                }
            }
        }
    }

    fn read_lexer_actions(&self, atn: &mut ATN, _data: &mut Iter<i32>) {
        //lexer actions are always supported here
        let nactions = *_data.next()?;

        for _i in 0..nactions {
            let action_type = *_data.next()?;

            let data1 = *_data.next()?;
            let data2 = *_data.next()?;

            let lexer_action = self.lexer_action_factory(action_type, data1, data2);

            atn.lexer_actions.push(lexer_action);
        }
    }

    fn mark_precedence_decisions(&self, _atn: &mut ATN, _data: &mut Iter<i32>) {
        let mut precedence_states = Vec::new();
        for state in _atn.states.iter() {
            if let ATNStateType::DecisionState {
                state: ATNDecisionState::StarLoopEntry { .. },
                ..
            } = state.get_state_type()
            {
                if let ATNStateType::RuleStartState {
                    is_left_recursive: true,
                    ..
                } = _atn.states
                    [_atn.rule_to_start_state[state.get_rule_index() as usize] as usize]
                    .get_state_type()
                {
                    let maybe_loop_end =
                        state.get_transitions().iter().last()?.get_target();
                    let maybe_loop_end = _atn.states[maybe_loop_end as usize].as_ref();
                    if let ATNStateType::LoopEndState(_) = maybe_loop_end.get_state_type() {
                        if maybe_loop_end.has_epsilon_only_transitions() {
                            if let ATNStateType::RuleStopState = _atn.states
                                [maybe_loop_end.get_transitions()[0].get_target() as usize]
                                .get_state_type()
                            {
                                precedence_states.push(state.get_state_number())
                            }
                        }
                    }
                }
            }
        }
        for st in precedence_states {
            if let ATNStateType::DecisionState {
                state:
                    ATNDecisionState::StarLoopEntry {
                        loop_back_state: _,
                        is_precedence,
                    },
                ..
            } = _atn.states[st as usize].get_state_type_mut()
            {
                *is_precedence = true
            }
        }
    }
}
