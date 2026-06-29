use std::{collections::{HashSet, VecDeque}, range::Range, slice::Iter};

use crate::{atn::{
        ATNStateRef, atn::{ATN, ATNType}, rule::ATNRule, state::{ATNDecisionState, ATNState, ATNStateType}, transition::Transition
    }, lex::LexerAction};

// If an error happens, check these for early returns on a parse error resulting in an empty Vec
pub fn read_states(data: &mut Iter<usize>) -> Option<Vec<ATNState>> {
    let mut states = Vec::new();
    let state_count = *data.next()?;

    for i in 0..state_count {
        let state_type = *data.next()?;
        
        // Is this even necessary?
        if state_type == 0 { continue }

        let rule_index = *data.next()?;

        let mut state = ATNState::new(state_type, rule_index, i)?;
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
        if let Some(s) = states.get_mut(state_index as usize) {
            s.set_nongreedy(true);
        }
    }

    let left_recursive_count = *data.next()?;
    for _ in 0..left_recursive_count {
        let state_index = *data.next()?;
        if let Some(s) = states.get_mut(state_index as usize) {
            s.set_left_recursive(true);
        }
    }

    Some(states)
}

/// This function only handles reading from a serialized array. The stop states for each rule
/// need to be modified by reading states, by finding each RuleStopState and adding it's value for the correct rule_index
pub fn read_rules(data: &mut Iter<usize>, lex: bool) -> Option<Vec<ATNRule>> {
    // Start states
    let nrules = *data.next()? as usize;

    let mut rules = Vec::new();

    for _ in 0..nrules {
        let start_state = *data.next()?;

        let rule = if lex {
            let token_type = *data.next()?;
            ATNRule::Lexer { start_state, stop_state: 0, token_type  }
        } else {
            ATNRule::Parser { start_state, stop_state: 0 }
        };

        rules.push(rule);
    }

    Some(rules)
}

pub fn read_modes(data: &mut Iter<usize>) -> Option<Vec<ATNStateRef>> {
    let mut modes = Vec::new();

    let nmodes = *data.next()?;
    for _ in 0..nmodes {
        modes.push(*data.next()?);
    }

    Some(modes)
}

pub fn read_sets(data: &mut Iter<usize>) -> Option<Vec<HashSet<usize>>> {
    // For now, instead of IntervalSet stuff, just add every item in the set to the HashSet manually
    let num_sets = *data.next()?;
    let mut sets = Vec::new();

    for _ in 0..num_sets {
        let mut set = HashSet::new();

        let num_intervals =  *data.next()?;
        if *data.next()? != 0 {
            // Contains EOF
            set.insert(usize::MAX);
        }

        for _ in 0..num_intervals {
            let begin = *data.next()?;
            let end = *data.next()?;

            (begin..=end).for_each(|value| { set.insert(value); });
        }

        sets.push(set);
    }

    Some(sets)
}

pub fn read_edges(data: &mut Iter<usize>) -> Option<Vec<Transition>> {
    let mut edges = Vec::new();
    let nedges = *data.next()?;

    for _i in 0..nedges {
        let x = [*data.next()?, *data.next()?, *data.next()?, *data.next()?, *data.next()?, *data.next()?];

        // necessary?
        if x[0] == 0 { continue }
        
        let transition = Transition::new(&mut x.iter())?;
        edges.push(transition);
    };

    Some(edges)
}

pub fn read_decisions(data: &mut Iter<usize>) -> Option<Vec<ATNStateRef>> {
    let mut decisions = Vec::new();

    let ndecisions = *data.next()?;
    for i in 0..ndecisions {
        decisions.push(*data.next()?);
    }
    
    Some(decisions)
}

pub fn read_lex_actions(data: &mut Iter<usize>) -> Option<Vec<LexerAction>> {

    let mut actions = Vec::new();
    let nactions = *data.next()?;

    for _ in 0..nactions {
        let action_type = *data.next()?;

        let arg1 = *data.next()?;
        let arg2 = *data.next()?;

        actions.push(LexerAction::new(action_type, arg1, arg2)?);
    }

    Some(actions)
}

fn mark_precedence_decisions(_atn: &mut ATN, _data: &mut Iter<usize>) {
    // let mut precedence_states = Vec::new();
    // for state in _atn.states.iter() {
    //     if let ATNStateType::DecisionState {
    //         state: ATNDecisionState::StarLoopEntry { .. },
    //         ..
    //     } = state.get_state_type()
    //     {
    //         if let ATNStateType::RuleStartState {
    //             is_left_recursive: true,
    //             ..
    //         } = _atn.states[_atn.rule_to_start_state[state.get_rule_index() as usize] as usize]
    //             .get_state_type()
    //         {
    //             let maybe_loop_end = state.transitions().iter().last()?.get_target();
    //             let maybe_loop_end = _atn.states[maybe_loop_end as usize].as_ref();
    //             if let ATNStateType::LoopEndState(_) = maybe_loop_end.get_state_type() {
    //                 if maybe_loop_end.has_epsilon_only_transitions() {
    //                     if let ATNStateType::RuleStopState = _atn.states
    //                         [maybe_loop_end.get_transitions()[0].get_target() as usize]
    //                         .get_state_type()
    //                     {
    //                         precedence_states.push(state.get_state_number())
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
    // for st in precedence_states {
    //     if let ATNStateType::DecisionState {
    //         state:
    //             ATNDecisionState::StarLoopEntry {
    //                 loop_back_state: _,
    //                 is_precedence,
    //             },
    //         ..
    //     } = _atn.states[st as usize].get_state_type_mut()
    //     {
    //         *is_precedence = true
    //     }
    // }
}
