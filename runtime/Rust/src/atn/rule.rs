use crate::atn::ATNStateRef;

pub enum ATNRule {
    Lexer {
        start_state: ATNStateRef,
        stop_state: ATNStateRef,
        token_type: usize
    },
    
    Parser {
        start_state: ATNStateRef,
        stop_state: ATNStateRef,
    }
}