mod atn;
mod config;
pub mod state;
mod parse;
pub mod transition;
mod rule;

pub use config::ATNConfig;
pub use config::ATNConfigType;

pub use atn::ATN;
pub use atn::ATNType;
// pub use state::ATNStateRef;

pub type ATNStateRef = usize;