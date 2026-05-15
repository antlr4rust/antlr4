use std::fmt::{Debug, Error, Formatter};
use std::ops::Deref;
use std::sync::Arc;

use crate::atn::ATN;
use crate::dfa::DFA;
use crate::prediction_context::PredictionContextCache;
use parking_lot::RwLock;

pub struct Simulator {
    pub atn: Arc<ATN>,
    pub shared_context_cache: Arc<PredictionContextCache>,
    pub decision_to_dfa: Arc<Vec<RwLock<DFA>>>,
}

impl Debug for Simulator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str("Simulator { .. }")
    }
}

impl Simulator {
    pub fn new(
        atn: Arc<ATN>,
        decision_to_dfa: Arc<Vec<RwLock<DFA>>>,
        shared_context_cache: Arc<PredictionContextCache>,
    ) -> Self {
        Self {
            atn,
            shared_context_cache,
            decision_to_dfa,
        }
    }

    fn shared_context_cache(&self) -> &PredictionContextCache {
        self.shared_context_cache.deref()
    }

    fn atn(&self) -> &ATN {
        self.atn.as_ref()
    }

    fn decision_to_dfa(&self) -> &Vec<RwLock<DFA>> {
        self.decision_to_dfa.as_ref()
    }
}
