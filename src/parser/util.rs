use std::vec::IntoIter;

use chumsky::input::Stream;
use logos::Logos as _;

use super::lexer;

/// Helper function (nostly for tests and benchmarks) to get Token stream out of input
pub fn lex(input: &str) -> Stream<IntoIter<lexer::Token<'_>>> {
    Stream::from_iter(
        lexer::Token::lexer(input)
            .map(|tok| tok.expect("expected a valid token."))
            .collect::<Vec<lexer::Token<'_>>>(),
    )
}

/// Produces a vector of tokens (for reuse or benchmarking).
pub fn tokenize(input: &str) -> Vec<lexer::Token<'_>> {
    lexer::Token::lexer(input).map(|tok| tok.unwrap()).collect()
}

/// Wraps tokens into a Stream that Chumsky can parse.
pub fn stream(tokens: Vec<lexer::Token<'_>>) -> Stream<IntoIter<lexer::Token<'_>>> {
    Stream::from_iter(tokens)
}
