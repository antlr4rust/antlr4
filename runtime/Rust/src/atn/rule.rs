use crate::atn::ATNStateRef;

pub struct Rule {
    start_state: ATNStateRef,
    stop_state: ATNStateRef,
    token_type: usize
}