use std::collections::HashSet;

use crate::atn::{transition::Transition, ATNStateRef, ATN};

impl ATN {
    pub fn closure(&self, current: ATNStateRef, input: usize) -> HashSet<ATNStateRef> {
        let mut states = self.epsilon_closure(current);
        states.insert(current);

        let mut closure = HashSet::new();

        let mut transitions = HashSet::new();
        for s in states.iter() {
            if let Some(state) = self.states.get(*s) {
                transitions = transitions.union(state.transitions()).copied().collect()
            }
        }

        for t in transitions {
            if let Some(transition) = self.transitions.get(t) {
                let target = transition.target();

                match transition {
                    Transition::Atom { label, .. } => {
                        if input == *label {
                            closure.insert(target);
                        }
                    }

                    Transition::Range { start, stop, .. } => {
                        if input > *start && input < *stop {
                            closure.insert(target);
                        }
                    }
                    _ => (), // Transition::Set {
                             //     set,
                             //     ..
                             // } => {

                             // },

                             // Transition::NotSet {
                             //     set,
                             //     ..
                             // } => {

                             // },

                             // Transition::Wildcard {
                             //     target: ATNStateRef,
                             // } => {

                             // },
                }
            }
        }

        // closure = closure.union(other).collect();

        closure
    }

    pub fn epsilon_closure(&self, current: ATNStateRef) -> HashSet<ATNStateRef> {
        let mut closure = HashSet::new();

        if let Some(state) = self.states.get(current) {
            for t in state.transitions() {
                if let Some(transition) = self.transitions.get(*t) {
                    if transition.is_epsilon() {
                        let mut sub_closure = self.epsilon_closure(transition.target());

                        closure.insert(transition.target());
                        closure = closure.union(&mut sub_closure).copied().collect();
                    }
                }
            }
        }

        closure
    }
}
