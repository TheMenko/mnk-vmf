use logos::Logos;

#[derive(Logos, Debug, Copy, Clone, PartialEq)]
pub enum Token<'a> {
    Error,

    #[regex(r#""([^"\\]|\\.)*""#, |lex| &lex.slice()[1..lex.slice().len()-1])]
    QuotedText(&'a str),

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident(&'a str),

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[token("{")]
    LBracket,

    #[token("}")]
    RBracket,

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
