use logos::Logos;

#[derive(Logos, Debug, Copy, Clone, PartialEq)]
pub enum Token<'a> {
    Error,

    #[token("\"")]
    Quote,

    //#[regex(r#"([^"\\]|\\.)*"#)]
    //Text,
    #[regex(r"[+-]?([0-9]*[.])?[0-9]+", priority = 1)]
    Float(&'a str),

    #[regex(r"[0-9]+", priority = 2)]
    Number(&'a str),

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

    #[regex(r"[ \t\f\n]+", logos::skip)]
    Whitespace,
}
