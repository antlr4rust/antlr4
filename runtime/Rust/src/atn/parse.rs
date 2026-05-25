use std::slice::Iter;

use crate::{
    atn::{
        atn::{ATN, ATNType}, rule::Rule, state::{ATNDecisionState, ATNState, ATNStateType}, transition::Transition
    },
    lex::IntervalSet,
};

fn read_states(data: &mut Iter<usize>) -> Vec<ATNState> {
    let mut states = Vec::new();
    let state_count = *data.next()?;

    for i in 0..state_count {
        let type_index = *data.next()?;
        let rule_index = *data.next()?;

        let mut state = ATNState::new(ATNStateType::new(type_index, rule_index), i);
        if let ATNStateType::LoopEndState(s) = &mut state.state_type {
            *s = *data.next()?
        }
    
        if let ATNStateType::DecisionState {
            state: ATNDecisionState::BlockStartState {
                end_state: s,
                .. 
            },
            ..
        } = &mut state.state_type {
            *s = *data.next()?
        };

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

fn read_rules(data: &mut Iter<i32>) -> Vec<Rule> {
    // Start states
    let nrules = *data.next()? as usize;

    for i in 0..nrules {
        let s = *data.next()?;
        atn.rule_to_start_state[i] = s;
        if atn.grammar_type == ATNType::LEXER {
            let token_type = *data.next()?;

            atn.rule_to_token_type.push(token_type);
        }
    }


    atn.rule_to_stop_state.resize(nrules, 0);
    for i in 0..atn.states.len() {
        let state = atn.states.get(i)?;
        if let ATNStateType::RuleStopState = state.get_state_type() {
            let rule_index = state.get_rule_index() as usize;
            atn.rule_to_stop_state[rule_index] = i as i32;
            let start_state = atn
                .states
                .get_mut(atn.rule_to_start_state[rule_index] as usize)?;
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

fn read_sets(data: &mut Iter<i32>) -> Vec<IntervalSet> {
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

fn read_edges(atn: &mut ATN, data: &mut Iter<i32>, sets: &Vec<IntervalSet>) {
    let states = Vec::new();
    let nedges = *data.next()?;

    for _i in 0..nedges {
        let src = *data.next()?;
        let trg = *data.next()?;
        let ttype = *data.next()?;
        let arg1 = *data.next()?;
        let arg2 = *data.next()?;
        let arg3 = *data.next()?;

        let transition = Transition::new(atn, ttype, src, trg, arg1, arg2, arg3, sets);

        atn.states.get_mut(src as usize)?.add_transition(transition);
    }

    let mut new_tr = Vec::new();
    for i in &atn.states {
        for tr in i.get_transitions() {
            match tr.get_serialization_type() {
                TransitionType::TRANSITION_RULE => {
                    let tr = tr.as_ref().cast::<RuleTransition>();
                    let target = atn.states.get(tr.get_target() as usize)?;

                    let outermost_prec_return = if let ATNStateType::RuleStartState {
                        is_left_recursive: true,
                        ..
                    } = atn
                        .states
                        .get(atn.rule_to_start_state[target.get_rule_index() as usize] as usize)?
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
            } => {}

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

fn mark_precedence_decisions(_atn: &mut ATN, _data: &mut Iter<i32>) {
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
            } = _atn.states[_atn.rule_to_start_state[state.get_rule_index() as usize] as usize]
                .get_state_type()
            {
                let maybe_loop_end = state.get_transitions().iter().last()?.get_target();
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
