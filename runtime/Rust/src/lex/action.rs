enum LexerActionType {
    Channel = 0,
    Custom,
    Mode,
    More,
    Pop,
    Push,
    Skip,
    Type,
}

impl LexerActionType {
    pub fn new(t: usize) -> Option<LexerActionType> {
        match t {
            0 => Some(Self::Channel),
            1 => Some(Self::Custom),
            2 => Some(Self::Mode),
            3 => Some(Self::More),
            4 => Some(Self::Pop),
            5 => Some(Self::Push),
            6 => Some(Self::Skip),
            7 => Some(Self::Type),
            _ => None
        }
    }
}

#[derive(Clone, Debug)]
pub enum LexerAction {
    Channel(usize),
    Custom {
        rule_index: usize,
        action_index: usize,
    },
    Mode(usize),
    More,
    Pop,
    Push(usize),
    Skip,
    Type(usize),
}

impl LexerAction {
    pub fn new(action_type: usize, arg1: usize, arg2: usize) -> Option<LexerAction> {
        let action_type = LexerActionType::new(action_type)?;

        Some(match action_type {
            LexerActionType::Channel => LexerAction::Channel(arg1),
            LexerActionType::Custom => LexerAction::Custom { rule_index: arg1, action_index: arg2 },
            LexerActionType::Mode => LexerAction::Mode(arg1),
            LexerActionType::More => LexerAction::More,
            LexerActionType::Pop => LexerAction::Pop,
            LexerActionType::Push => LexerAction::Push(arg1),
            LexerActionType::Skip => LexerAction::Skip,
            LexerActionType::Type => LexerAction::Type(arg1)
        })
    }
}