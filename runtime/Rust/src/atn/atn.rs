use std::collections::HashMap;
use std::io::Cursor;
use std::slice::Iter;

use std::fmt::{Debug, Formatter};

use crate::atn::atn_state::{ATNState, ATNStateRef, ATNStateType};
use crate::lex::token::TokenType;
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

        

        Ok(in_construction_atn)
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
            expected.remove_one(TokenType::Epsilon);
            // ctx = c.get_parent_ctx();
        }

        if following.contains(TokenType::Epsilon) {
            expected.add_one(TokenType::EOF);
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


   
}
