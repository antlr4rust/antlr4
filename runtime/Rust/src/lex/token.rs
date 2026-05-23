use std::borrow::Borrow;
use std::fmt::Formatter;
use std::fmt::{Debug, Display};

pub enum TokenType {
    Epsilon = -2,
    
    /// Type of tokens that DFA can use to advance to next state without consuming actual input token.
    /// Should not be created by downstream implementations.
    EOF = -1,
    Invalid = 0,
    
}

pub enum TokenChannel {
    Default,
    Hidden
}

#[derive(Debug)]
#[allow(missing_docs)]
pub struct Token<'input> {
    pub token_type: TokenType,
    pub channel: TokenChannel,
    pub start: usize,
    pub stop: usize,
    pub token_index: usize,
    pub line: usize,
    pub column: usize,
    pub text: &'input str,
    pub read_only: bool,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let txt = if self.token_type == TokenType::EOF {
            "<EOF>"
        } else {
            self.text.borrow()
        };
        let txt = txt.replace("\n", "\\n");
        let txt = txt.replace("\r", "\\r");
        let txt = txt.replace("\t", "\\t");
        //        let txt = escape_whitespaces(txt,false);
        f.write_fmt(format_args!(
            "[@{},{}:{}='{}',<{}>{},{}:{}]",
            self.get_token_index(),
            self.start,
            self.stop,
            txt,
            self.token_type,
            if self.channel > 0 {
                ",channel=".to_string() + self.channel.to_string().as_str()
            } else {
                String::new()
            },
            self.line,
            self.column
        ))
    }
}

impl Token<'_> {

    fn get_token_type(&self) -> TokenType {
        self.token_type
    }

    fn get_channel(&self) -> TokenChannel {
        self.channel
    }

    fn get_start(&self) -> usize {
        self.start
    }

    fn get_stop(&self) -> usize {
        self.stop
    }

    fn get_line(&self) -> usize {
        self.line
    }

    fn get_column(&self) -> usize {
        self.column
    }

    fn get_text(&self) -> &str {
        if self.token_type == TokenType::EOF {
            "<EOF>"
        } else {
            self.text.borrow()
        }
    }

    fn set_text(&mut self, _text: String) {
        unimplemented!()
    }

    fn get_token_index(&self) -> usize {
        self.token_index
    }

    fn set_token_index(&self, _v: usize) {
        self.token_index = _v
    }
}