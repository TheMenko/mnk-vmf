use logos::Logos;

#[derive(Logos, Debug, Copy, Clone, PartialEq)]
pub enum Token<'a> {
    Error,

    #[token("\"")]
    Quote,

    #[regex(
            r"(?x)
            [+-]?
            (?:[0-9][0-9_]*)
            (?:\.[0-9][0-9_]*)?
            (?:[eE][+-]?[0-9][0-9_]*)?
            ",
            |lex| lex.slice()
            , priority = 2)]
    Number(&'a str),

    #[regex(r#"[^ \t\f\r\n"()\[\]{}]+"#, |lex| lex.slice(), priority = 1)]
    Text(&'a str),

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    #[regex(r"[ \t\f\r\n]+", logos::skip)]
    Whitespace,
}

pub(crate) struct TokenIter<'a> {
    inner: logos::Lexer<'a, Token<'a>>,
}

impl<'a> TokenIter<'a> {
    pub fn new(input: &'a str) -> Self {
        TokenIter {
            inner: Token::lexer(input),
        }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Result<Token<'a>, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
