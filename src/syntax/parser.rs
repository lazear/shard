use std::collections::VecDeque;
use super::token::Token;

pub type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug)]
pub enum ParserError {
    Expecting(String),
    OutOfTokens
}

pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    /// Return a reference to the next token in the queue
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(0)
    }

    /// Is the next token equal to `expecting`
    pub fn peek_is(&self, expecting: &Token) -> bool {
        match self.peek() {
            Some(token) if token == expecting => true,
            _ => false,
        }
    }

    /// Mandatory pop
    /// Pop the next token off the queue
    pub fn pop(&mut self) -> ParserResult<Token> {
        self.tokens.pop_front().ok_or(ParserError::OutOfTokens)
    }

    /// Optional pop
    /// If the next token is equal to `expecting`, pop it and return true,
    /// otherwise return false and leave the next token  
    pub fn pop_if(&mut self, expecting: &Token) -> bool {
        let eq = self.peek_is(expecting);
        if eq {
            self.tokens.pop_front().expect("Impossible error");
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, expecting: &Token) -> ParserResult<Token> {
        let tok = self.pop()?;
        // We know tok is a token at this point, since the previous line
        // would've done an early return with ParserError::OutOfTokens otherwise
        if &tok == expecting {
            Ok(tok)
        } else {
            Err(ParserError::Expecting(format!("{:?}, found {:?}", tok, expecting)))
        }
    }

    pub fn expect_string(&mut self) -> ParserResult<Token> {
        let tok = self.pop()?;
        // We know tok is a token at this point, since the previous line
        // would've done an early return with ParserError::OutOfTokens otherwise
        match tok {
            Token::StringLiteral(_) => Ok(tok),
            _ => Err(ParserError::Expecting(format!("string, found {:?}", tok)))
        }
    }

    pub fn expect_number(&mut self) -> ParserResult<Token> {
        let tok = self.pop()?;
        // We know tok is a token at this point, since the previous line
        // would've done an early return with ParserError::OutOfTokens otherwise
        match tok {
            Token::NumberLiteral(_) => Ok(tok),
            _ => Err(ParserError::Expecting(format!("number, found {:?}", tok)))
        }
    }

    pub fn expect_identifier(&mut self) -> ParserResult<Token> {
        let tok = self.pop()?;
        // We know tok is a token at this point, since the previous line
        // would've done an early return with ParserError::OutOfTokens otherwise
        match tok {
            Token::Identifier(_) => Ok(tok),
            _ => Err(ParserError::Expecting(format!("identifier, found {:?}", tok)))
        }
    }

    pub fn from_tokens(v: Vec<Token>) -> Parser {
        Parser {
            tokens: VecDeque::from(v),
        }
    }
}