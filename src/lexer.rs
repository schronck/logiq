use crate::data::*;
use logos::Logos;

#[derive(Clone, Debug, Default, PartialEq, thiserror::Error)]
pub enum LexingError {
    #[error("Mismatching parentheses count")]
    ParenCountMismatch,
    #[error("{0} is not a valid logic gate")]
    NoSuchGate(String),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[default]
    #[error("Invalid token")]
    Other,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexingError> {
    let lex = Token::lexer(input);
    let mut tokens = vec![];
    let (mut open, mut close) = (0, 0);

    for token in lex {
        match token {
            Ok(Token::ParenOpen) => open += 1,
            Ok(Token::ParenClose) => close += 1,
            _ => {}
        };

        tokens.push(token?)
    }

    if open != close {
        return Err(LexingError::ParenCountMismatch);
    }

    Ok(tokens)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty() {
        let tokens = tokenize("").unwrap();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_paren_count_mismatch() {
        let err = Err(LexingError::ParenCountMismatch);

        assert_eq!(tokenize("("), err);
        assert_eq!(tokenize(")"), err);
    }

    #[test]
    fn test_terminal_id() {
        let tokens = tokenize("0").unwrap();
        assert_eq!(tokens, vec![Token::TerminalId(0)]);

        let tokens = tokenize("69").unwrap();
        assert_eq!(tokens, vec![Token::TerminalId(69)]);
    }

    #[test]
    fn test_gate() {
        let tokens = tokenize("and").unwrap();
        assert_eq!(tokens, vec![Token::Gate(Gate::And)]);

        let tokens = tokenize("or").unwrap();
        assert_eq!(tokens, vec![Token::Gate(Gate::Or)]);

        let tokens = tokenize("not").unwrap();
        assert_eq!(tokens, vec![Token::Gate(Gate::Not)]);

        let tokens = tokenize("nand").unwrap();
        assert_eq!(tokens, vec![Token::Gate(Gate::Nand)]);

        let tokens = tokenize("nor").unwrap();
        assert_eq!(tokens, vec![Token::Gate(Gate::Nor)]);

        let tokens = tokenize("xor").unwrap();
        assert_eq!(tokens, vec![Token::Gate(Gate::Xor)]);
    }

    #[test]
    fn test_logic_string() {
        let tokens = tokenize("(0 and 1)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::ParenOpen,
                Token::TerminalId(0),
                Token::Gate(Gate::And),
                Token::TerminalId(1),
                Token::ParenClose
            ]
        );

        let tokens = tokenize("((0 and 1) or 2)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::ParenOpen,
                Token::ParenOpen,
                Token::TerminalId(0),
                Token::Gate(Gate::And),
                Token::TerminalId(1),
                Token::ParenClose,
                Token::Gate(Gate::Or),
                Token::TerminalId(2),
                Token::ParenClose
            ]
        );
    }
}
