pub mod token;
mod vocabulary;
mod lexer;
mod lexer_atn_simulator;
mod interval_set;

pub use lexer::Lexer;

pub use interval_set::IntervalSet;
pub use interval_set::Interval;