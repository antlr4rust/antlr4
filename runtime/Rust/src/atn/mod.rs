mod atn;
pub mod state;
mod parse;
pub mod transition;
mod rule;
mod sim;

pub use atn::ATN;
pub use atn::ATNType;
pub use state::ATNState;

pub type ATNStateRef = usize;
pub type ATNRuleRef = usize;
pub type ATNTransitionRef = usize;