#![allow(dead_code)]
use std::string::String;
use std::mem;
use self::Token::*;

type LexerResult<T> = Result<T, String>;

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

#[derive(Debug, PartialEq, Clone)]
enum State {
    None,
    Text,
    Number,
    Disambiguate,
    Comment,
    Operator,
    Escape(bool),
}

#[derive(Debug)]
struct Lexer {
    tokens: Vec<Token>,
    last_char: char,
    buffer: String,
    state: State,
    line: usize,
    column: usize,
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
            "order" => ORDER,
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
            '`' => Ok(State::Escape(false)),
            // Whitespace, return None
            ' ' | '\t' | '\n' => Ok(State::None),
            // Other UTF-8 character
            _ => {
                let s = "<>|-+()[].,;*&|/=";
                if let Some(index) = s.find(c) {
                    if index < 4 {
                        Ok(State::Disambiguate)
                    } else {
                        Ok(State::Operator)
                    }
                } else {
                    Err(format!("Illegal character {} on line {}, column {} found.", c, self.line, self.column))
                }

            }
        }
    }

    fn feed(&mut self, c: char) -> LexerResult<State> {
        // Update line and column number
        if c == '\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        };

        let state = match self.state {
            // Current state is comment, switch to None if newline
            State::Comment => {
                if c == '\n' {
                    State::None
                } else {
                    State::Comment
                }
            },
            // Current state is none, so we are at the beginning, or whitespace
            State::None => {
                let next = self.next_state(c)?;
                match next {
                    State::None | State::Comment => (),
                    State::Text | State::Number => {
                        self.buffer.push(c);
                    }
                    State::Operator => {
                        self.tokens.push(Token::from_char(c).expect(&format!("Illegal character {} on line {}, column {} found. Expected valid operator", c, self.line, self.column)));
                    }
                    _ => (),
                };
                next
            },
            // Current state is text, so we are reading a string
            State::Text => {
                match self.next_state(c)? {
                    State::Text | State::Number => {
                        self.buffer.push(c);
                        State::Text
                    },
                    State::None => {
                        let word: String = mem::replace(&mut self.buffer, String::new());
                        self.tokens.push(Token::from_str(&word));
                        State::None
                    }
                    // Invalid character
                    _ => return Err(format!("Illegal character '{}' on line {}, column {} found. Expected valid identifier, a-Z|0-9|_", c, self.line, self.column))
                }
            },

            // Current state is number, so only acceptable chars are 0-9 and '.'
            State::Number => {
                match self.next_state(c)? {
                    State::Number => {
                        self.buffer.push(c);
                        State::Number
                    },
                    State::Operator => {
                        if c == '.' {
                            self.buffer.push(c);
                            State::Number
                        } else {
                            return Err(format!("Illegal character {} on line {}, column {} found. Expected valid number, 0-9.", c, self.line, self.column))
                        }
                    },
                    _ => return Err(format!("Illegal character {} on line {}, column {} found. Expected valid number, 0-9.", c, self.line, self.column))
                }
            },

            State::Escape(escaped) => {
                match (escaped, c) {
                    (false, '`') => State::Escape(false),
                    (true, '`') => {
                        let word: String = mem::replace(&mut self.buffer, String::new());
                        self.tokens.push(Token::StringLiteral(word));
                        State::None
                    },
                    // Any character, any combination
                    (_, _) => {
                        self.buffer.push(c);
                        State::Escape(true)
                    },
                }
            },
            
            State::Disambiguate => {
                match (self.last_char, c) {
                    ('-', '-') => State::Comment,
                    ('<', '>') => {
                        self.tokens.push(Token::NOTEQUAL);
                        State::None
                    },
                    (_, _) => State::None
                }
            },

            State::Operator => {
                State::None
            },
        };
        self.state = state.clone();
        self.last_char = c;
        Ok(state)        
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn from_char() {
        // Valid character
        assert_eq!(Token::from_char('.'), Some(Token::DOT));
        // Invalid character
        assert_eq!(Token::from_char('#'), None);
    }

    #[test]
    fn from_str() {
        // Everything is case insensitive
        assert_eq!(Token::from_str("SEleCT"), Token::SELECT);
        assert_eq!(Token::from_str("DROP"), Token::DROP);
        // All identifiers are stored as lower case
        assert_eq!(Token::from_str("User_id"), Token::Identifier("user_id".into()));
    }

    #[test]
    /// Test state transitions from State::None -> State::_
    fn next_state() {
        let lex = Lexer {
            state: State::None,
            tokens: Vec::new(),
            last_char: ' ',
            buffer: String::new(),
            column: 0,
            line: 0,
        };
        assert_eq!(lex.next_state('.'), Ok(State::Operator));
        assert_eq!(lex.next_state('a'), Ok(State::Text));
        assert_eq!(lex.next_state('9'), Ok(State::Number));
        assert_eq!(lex.next_state('`'), Ok(State::Escape(false)));
        assert_eq!(lex.next_state('<'), Ok(State::Disambiguate));
        assert_eq!(lex.next_state('='), Ok(State::Operator));
    }

    #[test]
    /// Test lexing of an identifier
    fn feed_identifier() {
        let mut lex = Lexer {
            state: State::None,
            tokens: Vec::new(),
            last_char: ' ',
            buffer: String::new(),
            column: 0,
            line: 0,
        };

        let s = "my_table";
        for c in s.chars() {
            assert_eq!(lex.feed(c), Ok(State::Text));
        }
        assert_eq!(lex.feed(' '), Ok(State::None));
        assert_eq!(lex.tokens.pop(), Some(Token::Identifier(s.into())));
        assert_eq!(lex.state, State::None);
    }

    #[test]
    /// Test lexing of a literal
    fn feed_literal() {
        let mut lex = Lexer {
            state: State::None,
            tokens: Vec::new(),
            last_char: ' ',
            buffer: String::new(),
            column: 0,
            line: 0,
        };
        
        // Try lexing a string literal
        assert_eq!(lex.feed('`'), Ok(State::Escape(false)));
        for c in "user_id".chars() {
            assert_eq!(lex.feed(c), Ok(State::Escape(true)));
        };
        assert_eq!(lex.feed('`'), Ok(State::None));
        assert_eq!(lex.tokens.pop(), Some(Token::StringLiteral("user_id".into())));
        assert_eq!(lex.column, 9);
    }

    #[test]
    fn feed_comment() {
        let mut lex = Lexer {
            state: State::None,
            tokens: Vec::new(),
            last_char: ' ',
            buffer: String::new(),
            column: 0,
            line: 0,
        };
        assert_eq!(lex.feed('-'), Ok(State::Disambiguate));
        assert_eq!(lex.feed('-'), Ok(State::Comment));
        for c in "line comment".chars() {
            assert_eq!(lex.feed(c), Ok(State::Comment));
        };
        assert_eq!(lex.feed('\n'), Ok(State::None));
    }

    #[test]
    fn feed_statement() {
        let mut lex = Lexer {
            state: State::None,
            tokens: Vec::new(),
            last_char: ' ',
            buffer: String::new(),
            column: 0,
            line: 0,
        };

        let query = "SELECT * FROM my_table WHERE name = `user1`";
        for c in query.chars() {
            lex.feed(c).unwrap();
        }
        assert_eq!(lex.tokens.pop(), Some(Token::StringLiteral("user1".into())));
        assert_eq!(lex.tokens.pop(), Some(Token::EQUAL));
        assert_eq!(lex.tokens.pop(), Some(Token::Identifier("name".into())));
        assert_eq!(lex.tokens.pop(), Some(Token::WHERE));
        assert_eq!(lex.tokens.pop(), Some(Token::Identifier("my_table".into())));
        assert_eq!(lex.tokens.pop(), Some(Token::FROM));
        assert_eq!(lex.tokens.pop(), Some(Token::ASTERISK));
        assert_eq!(lex.tokens.pop(), Some(Token::SELECT));
    }
}
