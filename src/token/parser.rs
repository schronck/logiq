use super::{ParsedTokens, ScannedTokens, Token};
use crate::gate::Gate;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("the resulting expression has dangling terminals")]
    InvalidExpression,
    #[error("two consecutive terminals in expression")]
    InvalidTerminalPlacement,
    #[error("gates must come between terminals")]
    InvalidGatePlacement,
    #[error("invalid token: {0}")]
    InvalidToken(char),
    #[error("missing on mangled parentheses in expression")]
    InvalidParentheses,
    #[error("parsed an empty expression")]
    EmptyExpression,
    #[error("{0}")]
    ScannerError(#[from] super::scanner::ScannerError),
    #[error(transparent)]
    Transparent(#[from] anyhow::Error),
}

pub fn parse(tokens: &ScannedTokens) -> Result<ParsedTokens, ParserError> {
    let mut parsed_tokens = Vec::<Token>::new();
    let mut parentheses = 0_usize;
    let dangling_terminal = parse_tokens_from_iter(
        &mut parentheses,
        &mut parsed_tokens,
        &mut tokens.tokens().iter(),
    )?;

    if parsed_tokens.is_empty() {
        Err(ParserError::EmptyExpression)
    } else if parentheses != 0 {
        Err(ParserError::InvalidParentheses)
    } else if !dangling_terminal || parsed_tokens.len() == 1 {
        // there might be a dangling terminal but if it's the only thing
        // parsed, then it's fine (i.e. it's a single requirement)
        Ok(ParsedTokens(parsed_tokens))
    } else {
        Err(ParserError::InvalidExpression)
    }
}

fn parse_tokens_from_iter<'a, I>(
    parentheses: &mut usize,
    parsed_tokens: &mut Vec<Token>,
    tokens: &mut I,
) -> Result<bool, ParserError>
where
    I: Iterator<Item = &'a Token>,
{
    let mut dangling_terminal = false;
    while let Some(token) = tokens.next() {
        match token {
            Token::Whitespace => continue, // skip
            Token::OpeningParenthesis => {
                *parentheses = parentheses.saturating_add(1);
                parsed_tokens.push(*token);
                dangling_terminal = parse_tokens_from_iter(parentheses, parsed_tokens, tokens)?
            }
            Token::ClosingParenthesis => {
                *parentheses = parentheses.wrapping_sub(1);
                if last_matches_variant(parsed_tokens, &Token::OpeningParenthesis) {
                    parsed_tokens.pop(); // pop last (
                } else if last_matches_variant(parsed_tokens, &Token::Gate(Gate::And)) {
                    return Err(ParserError::InvalidGatePlacement);
                } else {
                    parsed_tokens.push(*token);
                }
                return Ok(dangling_terminal);
            }
            Token::Terminal(_) => {
                if last_matches_variant(parsed_tokens, &Token::Terminal('_'))
                    || last_matches_variant(parsed_tokens, &Token::ClosingParenthesis)
                {
                    return Err(ParserError::InvalidTerminalPlacement);
                }
                dangling_terminal = !dangling_terminal;
                parsed_tokens.push(*token);
            }
            Token::Gate(_) => {
                if !(last_matches_variant(parsed_tokens, &Token::Terminal('_'))
                    || last_matches_variant(parsed_tokens, &Token::ClosingParenthesis))
                {
                    return Err(ParserError::InvalidGatePlacement);
                } else if !dangling_terminal {
                    dangling_terminal = true;
                }
                parsed_tokens.push(*token);
            }
        }
    }
    Ok(dangling_terminal)
}

fn last_matches_variant(tokens: &[Token], token: &Token) -> bool {
    if tokens.is_empty() {
        false
    } else {
        is_equal_discriminant(&tokens[tokens.len() - 1], token)
    }
}

fn is_equal_discriminant<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_single_variable() {
        let scanned = ScannedTokens::from_str("p").unwrap();
        let parsed = parse(&scanned).unwrap();
        assert_eq!(parsed.tokens(), &[Token::Terminal('p')]);
        // NOTE the parser is not smart enough to accept these as
        // valid inputs
        let scanned = ScannedTokens::from_str("(p)").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidExpression,
        ));
        let scanned = ScannedTokens::from_str("((((p))))").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidExpression,
        ));
    }

    #[test]
    fn parse_single_whitespace() {
        let scanned = ScannedTokens::from_str(" ").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::EmptyExpression
        ));
    }

    #[test]
    fn parse_invalid_parentheses() {
        let scanned = ScannedTokens::from_str("( )").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::EmptyExpression,
        ));

        let scanned = ScannedTokens::from_str("(()").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidParentheses,
        ));

        let scanned = ScannedTokens::from_str("    )").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidParentheses,
        ));

        let scanned = ScannedTokens::from_str("(())(").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidParentheses,
        ));

        let scanned = ScannedTokens::from_str("())))))))))))))").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidParentheses,
        ));
    }

    #[test]
    fn parse_invalid_terminals() {
        let scanned = ScannedTokens::from_str("a b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidTerminalPlacement
        ));

        let scanned = ScannedTokens::from_str("(a)").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidExpression
        ));

        let scanned = ScannedTokens::from_str("(a AND c) b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidTerminalPlacement
        ));

        let scanned = ScannedTokens::from_str("(a) b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidTerminalPlacement
        ));
    }

    #[test]
    fn parse_invalid_gates() {
        let scanned = ScannedTokens::from_str("a AND OR b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        let scanned = ScannedTokens::from_str("(a AND) OR b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        let scanned = ScannedTokens::from_str("(a AND OR) b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        let scanned = ScannedTokens::from_str("a AND (OR) b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        let scanned = ScannedTokens::from_str("a AND () b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidExpression
        ));

        let scanned = ScannedTokens::from_str("a AND ( ()) b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidExpression
        ));
    }

    #[test]
    fn parse_valid_statement() {
        let scanned = ScannedTokens::from_str("a AND b OR c").unwrap();
        let parsed = parse(&scanned).unwrap();
        assert_eq!(
            parsed.tokens(),
            &[
                Token::Terminal('a'),
                Token::Gate(Gate::And),
                Token::Terminal('b'),
                Token::Gate(Gate::Or),
                Token::Terminal('c')
            ]
        );
        let scanned = ScannedTokens::from_str("a AND (b OR c) XOR d").unwrap();
        let parsed = parse(&scanned).unwrap();
        assert_eq!(
            parsed.tokens(),
            &[
                Token::Terminal('a'),
                Token::Gate(Gate::And),
                Token::OpeningParenthesis,
                Token::Terminal('b'),
                Token::Gate(Gate::Or),
                Token::Terminal('c'),
                Token::ClosingParenthesis,
                Token::Gate(Gate::Xor),
                Token::Terminal('d')
            ]
        );
    }
}
