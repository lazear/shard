//! Lexical analysis module

#![allow(dead_code)]
use std::string::String;
use std::mem;
use super::token::*;
use super::parser::Parser;

type LexerResult<T> = Result<T, String>;

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
/// Finite state machine for lexical analysis of queries
pub struct Lexer {
    // List of tokens we have parsed
    tokens: Vec<Token>,
    // Last read character
    last_char: char,
    // Word/number we are currently lexing
    buffer: String,
    // Current lexer state
    state: State,
    // Line number
    line: usize,
    // Column number
    column: usize,
}

impl Lexer {
    /// Return an error message
    fn error(&self, c: char, expected: &str) -> LexerResult<State> {
        Err(format!(
            "Illegal character `{}` encountered on line {}, \
            column {} during lexical analysis. Expected {}",
            c,
            self.line,
            self.column,
            expected
        ))
    }

    /// Retrieve the last lexed token
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
            'a'...'z' | 'A'...'Z' | '_' => Ok(State::Text),
            // numbers must start with a number...
            '0'...'9' => Ok(State::Number),
            // Literals must start with a single apostrophe
            '`' => Ok(State::Escape(false)),
            // Whitespace, return None
            ' ' | '\t' | '\n' => Ok(State::None),
            // Other UTF-8 character
            ';' => Ok(State::None),
            _ => {
                let s = "<>|-+()[].,;*&|/=";
                if let Some(index) = s.find(c) {
                    if index < 4 {
                        Ok(State::Disambiguate)
                    } else {
                        Ok(State::None)
                    }
                } else {
                    self.error(c, "<>|-+()[].,*&|/=")
                }

            }
        }
    }

    /// Feed a character into the lexer. Finite state machine
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
            }
            // Current state is none, so we are at the beginning, or whitespace
            State::None => {
                let next = self.next_state(c)?;
                match next {
                    State::None => {
                        if let Some(tok) = Token::from_char(c) {
                            self.tokens.push(tok);
                        }
                    }
                    State::Comment => (),
                    State::Text | State::Number => {
                        self.buffer.push(c);
                    }
                    State::Operator => {
                        self.tokens.push(Token::from_char(c).expect(&format!(
                            "Illegal character `{}` encountered on line {},\
                                 column {} during lexical analysis. Expected valid operator",
                            c,
                            self.line,
                            self.column
                        )));
                    }
                    _ => (),
                };
                next
            }
            // Current state is text, so we are reading a string
            State::Text => {
                match self.next_state(c)? {
                    // Continue reading into buffer
                    State::Text | State::Number => {
                        self.buffer.push(c);
                        State::Text
                    }
                    // Whitespace
                    State::None => {
                        let word: String = mem::replace(&mut self.buffer, String::new());
                        self.tokens.push(Token::from_str(&word));
                        if let Some(tok) = Token::from_char(c) {
                            self.tokens.push(tok);
                        }
                        State::None
                    }
                    // Invalid character
                    _ => return self.error(c, "valid identifier [a-Z|0-9|_]"),
                }
            }

            // Current state is number, so only acceptable chars are 0-9 and '.'
            State::Number => {
                match self.next_state(c)? {
                    // Continue reading into buffer
                    State::Number => {
                        self.buffer.push(c);
                        State::Number
                    }
                    // Check for decimal place
                    State::Operator => {
                        if c == '.' {
                            self.buffer.push(c);
                            State::Number
                        } else {
                            // Invalid character
                            return self.error(c, "valid number [0-9|.]");
                        }
                    }
                    State::None => {
                        let word: String = mem::replace(&mut self.buffer, String::new());
                        self.tokens.push(Token::NumberLiteral(word));
                        if let Some(tok) = Token::from_char(c) {
                            self.tokens.push(tok);
                        }
                        State::None
                    }
                    // Invalid character
                    _ => return self.error(c, "valid number [0-9|.]"),
                }
            }
            // Reading literals, any UTF-8 character is valid except for backtick
            State::Escape(escaped) => {
                match (escaped, c) {
                    (false, '`') => State::Escape(false),
                    // This is a closing backtick
                    (true, '`') => {
                        // Was the backtick escaped? If not, then save the token
                        if self.last_char != '\\' {
                            let word: String = mem::replace(&mut self.buffer, String::new());
                            self.tokens.push(Token::StringLiteral(word));
                        }
                        State::None
                    }
                    // Any character, any combination
                    (_, _) => {
                        self.buffer.push(c);
                        State::Escape(true)
                    }
                }
            }
            // Operator or character that needs disambiguation
            State::Disambiguate => {
                match (self.last_char, c) {
                    ('-', '-') => State::Comment,
                    ('<', '>') => {
                        self.tokens.push(Token::NOTEQUAL);
                        State::None
                    }
                    ('<', '=') => {
                        self.tokens.push(Token::LESSTHANOREQUAL);
                        State::None
                    }
                    ('>', '=') => {
                        self.tokens.push(Token::GREATERTHANOREQUAL);
                        State::None
                    }
                    ('>', ' ') => {
                        self.tokens.push(Token::GREATERTHAN);
                        State::None
                    }
                    ('<', ' ') => {
                        self.tokens.push(Token::LESSTHAN);
                        State::None
                    }
                    ('|', '|') => {
                        self.tokens.push(Token::DOUBLEPIPE);
                        State::None
                    }
                    (_, _) => return self.error(c, "matching operator"),
                }
            }
            // Operator. Token was already pushed, transition back to none
            State::Operator => State::None,
        };
        // Save state, and last character
        self.state = state.clone();
        self.last_char = c;
        Ok(state)
    }

    pub fn lex(s: &str) -> LexerResult<Parser> {
        let mut lex = Lexer {
            state: State::None,
            tokens: Vec::new(),
            last_char: ' ',
            buffer: String::new(),
            column: 0,
            line: 0,
        };

        for c in s.chars() {
            lex.feed(c)?;
        }
        Ok(Parser::from_tokens(lex.tokens))
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_str() {
        let mut parser = Lexer::lex("select * from my_table where row_id > 0;").unwrap();
        let v = vec![
            Token::SELECT,
            Token::ASTERISK,
            Token::FROM,
            Token::Identifier("my_table".into()),
            Token::WHERE,
            Token::Identifier("row_id".into()),
            Token::GREATERTHAN,
            Token::NumberLiteral("0".into()),
            Token::SEMICOLON,
        ];
        for tok in v.into_iter() {
            parser.expect(&tok).unwrap();
        }
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
        assert_eq!(lex.next_state('.'), Ok(State::None));
        assert_eq!(lex.next_state('a'), Ok(State::Text));
        assert_eq!(lex.next_state('9'), Ok(State::Number));
        assert_eq!(lex.next_state('`'), Ok(State::Escape(false)));
        assert_eq!(lex.next_state('<'), Ok(State::Disambiguate));
        assert_eq!(lex.next_state('='), Ok(State::None));
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
        }
        assert_eq!(lex.feed('`'), Ok(State::None));
        assert_eq!(
            lex.tokens.pop(),
            Some(Token::StringLiteral("user_id".into()))
        );
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
        }
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
