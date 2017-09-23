//! Lexical analysis and parsing of SQL syntax
//!
//! Shard uses a simplified SQL syntax, derived from SQLite and Postgres

pub mod lexer;
pub mod token;
pub mod parser;