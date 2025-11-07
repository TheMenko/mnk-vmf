use std::vec::IntoIter;

use chumsky::input::Stream;
use logos::Logos as _;

use super::lexer;

/// Macro to define individual property parsers and combine them with .or().
/// When this Macro is used, it is necrssary to have chumsky's .or() and .map() in the scope.
///
/// Usage:
/// ```ignore
/// impl_block_properties_parser! {
///     // Output variable name for the combined parser: Its type will be Box<dyn Parser<..., Output = $PropEnumType, ...>>
///     any_property_variable_name: YourPropertyEnumType = {
///         // var_name = parser_call_expr => enum_variant_constructor_or_mapper_fn
///         p_some_bool = some_parser_that_outputs_bool("some_key") => YourPropertyEnumType::SomeBool,
///         p_some_num  = some_parser_that_outputs_u32("num_key")  => YourPropertyEnumType::SomeNum,
///         // ...
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_block_properties_parser {
    (@build_or_chain $first_parser_var:ident) => {
        $first_parser_var
    };
    (@build_or_chain $first_parser_var:ident, $($rest_parser_vars:ident),+) => {
        $first_parser_var.or(impl_block_properties_parser!(@build_or_chain $($rest_parser_vars),+))
    };

    (
        $any_property_let_name:ident: $PropEnumType:ty = {
            $(
                $var_name:ident = $parser_call_expr:expr => $value_mapper_fn:expr
            ),+ $(,)? // Allow trailing comma
        }
    ) => {
        $(
            let $var_name = $parser_call_expr.map($value_mapper_fn);
        )+
        let $any_property_let_name =
            impl_block_properties_parser!(@build_or_chain $($var_name),+).boxed();
    };
}

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
    lexer::Token::lexer(input)
        .enumerate()
        .map(|(idx, tok)| {
            tok.unwrap_or_else(|_| {
                let lexer = lexer::Token::lexer(input);
                let span = lexer.span();
                let context = &input
                    [span.start.saturating_sub(20)..span.end.saturating_add(20).min(input.len())];
                panic!(
                    "Failed to tokenize at position {} (token #{})\nContext: {:?}",
                    span.start, idx, context
                );
            })
        })
        .collect()
}

/// Wraps tokens into a Stream that Chumsky can parse.
pub fn stream(tokens: Vec<lexer::Token<'_>>) -> Stream<IntoIter<lexer::Token<'_>>> {
    Stream::from_iter(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let input = r#"world
    {
        "id" "1"
    }"#;
        let tokens = tokenize(input);
        println!("Tokens: {:#?}", tokens);
    }

    #[test]
    fn test_tokenize_with_special_chars() {
        let input = r#""detail/detailsprites""#;
        let tokens = tokenize(input);
        println!("Tokens: {:#?}", tokens);
    }

    #[test]
    fn test_tokenize_with_brackets() {
        let input = r#""[1 0 0 0]""#;
        let tokens = tokenize(input);
        println!("Tokens: {:#?}", tokens);
    }

    #[test]
    fn test_tokenize_negative_number() {
        let input = r#""-1""#;
        let tokens = tokenize(input);
        println!("Tokens: {:#?}", tokens);
    }
}
