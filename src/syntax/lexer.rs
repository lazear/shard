#![allow(dead_code)]
use std::string::String;
use std::mem;
use self::Token::*;

type LexerResult<T> = Result<T, char>;

/// Lexical token
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    // keywords
    SELECT,
    FROM,
    WHERE,
    ORDER,
    INSERT,
    INTO,
    CREATE,
    TABLE,
    DROP,
    IF,
    NOT,
    NULL,
    DEFAULT,
    SERIAL,

    // types
    INTEGER,
    TEXT,
    FLOAT,
    BLOB,

    // operators
    EQUAL,
    NOTEQUAL,
    LESSTHAN,
    LESSTHANOREQUAL,
    GREATERTHAN,
    GREATERTHANOREQUAL,
    PLUS,
    MINUS,
    FORWARDSLASH,

    // blocks
    LEFTPAREN,
    RIGHTPAREN,
    LEFTBRACKET,
    RIGHTBRACKET,

    // other
    DOT,
    COMMA,
    SEMICOLON,
    ASTERISK,
    AMPERSAND,
    PIPE,
    DOUBLEPIPE,
    
    // literals
    StringLiteral(String),
    NumberLiteral(String),
    Identifier(String),
}

enum State {
    None,
    Text,
    Number,
    Disambigaute,
    Comment,
    Escape(bool),
}

struct Lexer {
    tokens: Vec<Token>,
    last_char: char,
    buffer: String,
    state: State,
}

impl Token {
    fn from_char(c: char) -> Option<Token> {
        Some(match c {
            '=' => EQUAL,
            '<' => LESSTHAN,
            '>' => GREATERTHAN,
            '+' => PLUS,
            '-' => MINUS,
            '(' => LEFTPAREN,
            '[' => LEFTBRACKET,
            ')' => RIGHTPAREN,
            ']' => RIGHTBRACKET,
            '.' => DOT,
            ',' => COMMA,
            ';' => SEMICOLON,
            '*' => ASTERISK,
            '&' => AMPERSAND,
            '|' => PIPE,
            '/' => FORWARDSLASH,       
            _ => return None,
        })
    }

    fn from_str(s: &str) -> Token {
        let word: String = s.chars().flat_map(|c| c.to_lowercase()).collect();
        match word.as_ref() {
            "select" => SELECT,
            "from" => FROM,
            "where" => WHERE,
            "insert" => INSERT,
            "into" => INTO,
            "create" => CREATE,
            "table" => TABLE,
            "drop" => DROP,
            "if" => IF,
            //"exists" => EXISTS,
            "not" => NOT,
            "null" => NULL,
            "default" => DEFAULT,
            "int" | "integer" => INTEGER,
            "text" => TEXT,
            "float" => FLOAT,
            "blob" => BLOB,
            "serial" => SERIAL,
            _ => Identifier (word),
        } 
    }
}

impl Lexer {
    fn last_token(&self) -> Option<Token> {
        if self.tokens.len() > 0 {
            Some(self.tokens[self.tokens.len() - 1].clone())
        } else {
            None
        }        
    }

    /// Transition to the next state from State::None 
    fn next_state(&self, c: char) -> LexerResult<State> {
        match c {
            // Identifiers and keywords must start with a letter or underscore
            'a' ... 'z' | 'A' ... 'Z' | '_' => Ok(State::Text),
            // numbers must start with a number...
            '0'...'9' => Ok(State::Number),
            // Literals must start with a single apostrophe
            '\'' => Ok(State::Escape(false)),
            // Whitespace
            ' ' | '\t' | '\n' => Ok(State::None),
            // Other UTF-8 character
            _ => {
                let s = "<>|-+()[].,;*&|/";
                if let Some(index) = s.find(c) {
                    if index < 4 {
                        Ok(State::Disambigaute)
                    } else {
                        Ok(State::None)
                    }
                } else {
                    Err(c)
                }

            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn from_char() {
        assert_eq!(Token::from_char('.'), Some(Token::DOT));
        assert_eq!(Token::from_char('#'), None);
    }

    #[test]
    fn from_str() {
        assert_eq!(Token::from_str("SEleCT"), Token::SELECT);
        assert_eq!(Token::from_str("DROP"), Token::DROP);
        // All identifiers are lower case
        assert_eq!(Token::from_str("User_id"), Token::Identifier("user_id".into()));
    }
}
