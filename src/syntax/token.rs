//! Lexical tokens
use self::Token::*;

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
    EXISTS,
    NOT,
    NULL,
    DEFAULT,
    SERIAL,
    AND,
    OR,

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

impl Token {
    /// Match a character into a token
    pub fn from_char(c: char) -> Option<Token> {
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

    /// Match a word into either a keyword, or assume it is an identifier
    pub fn from_str(s: &str) -> Token {
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
            "exists" => EXISTS,
            "not" => NOT,
            "null" => NULL,
            "default" => DEFAULT,
            "serial" => SERIAL,
            "and" => AND,
            "or" => OR,
            "int" | "integer" => INTEGER,
            "text" => TEXT,
            "float" => FLOAT,
            "blob" => BLOB,
            _ => Identifier(word),
        }
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
        assert_eq!(
            Token::from_str("User_id"),
            Token::Identifier("user_id".into())
        );
    }
}
