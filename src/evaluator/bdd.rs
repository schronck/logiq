use super::TerminalId;
use crate::gate::Gate;
use crate::token::{ParsedTokens, Token};

use boolean_expression::{BDDFunc, BDD};

/// Builds a BDD (Binary Decision Diagram) from a parsed token stream.
///
/// # Example
/// ```
/// # use requiem::evaluator::build_bdd;
/// # use requiem::token::ParsedTokens;
/// # use boolean_expression::{BDDFunc, BDD};
/// # use std::collections::HashMap;
/// # use std::str::FromStr;
///
/// let mut bdd = BDD::new();
/// let tokens = ParsedTokens::from_str("x AND y OR z").unwrap();
/// let bdd_func = build_bdd(&mut bdd, &tokens).unwrap();
///
/// // NOTE bdd.labels() returns labels in a random order
/// assert_eq!(bdd.labels().len(), 3);
/// assert!(bdd.labels().contains(&'x'));
/// assert!(bdd.labels().contains(&'y'));
/// assert!(bdd.labels().contains(&'z'));
///
/// let mut evals = HashMap::new();
/// evals.insert('x', true);
/// evals.insert('y', false);
/// evals.insert('z', true);
/// assert!(bdd.evaluate(bdd_func, &evals));
///
/// let mut evals = HashMap::new();
/// evals.insert('x', true);
/// evals.insert('y', false);
/// evals.insert('z', false);
/// assert!(!bdd.evaluate(bdd_func, &evals));
/// ```
///
/// Works with (almost arbitrary) compound logic, e.g. `"((a OR b) OR (c AND
/// d)) XOR a"`. It only accepts [`ParsedTokens`] as input, therefore it's hard
/// to misuse it with invalid input.
pub fn build_bdd(
    bdd: &mut BDD<TerminalId>,
    parsed_tokens: &ParsedTokens,
) -> Result<BDDFunc, String> {
    build_bdd_from_iter(bdd, &mut parsed_tokens.tokens().iter())
        .ok_or("invalid logic: BDDFunc cannot be None".to_owned())
}

fn build_bdd_from_iter<'a, I>(bdd: &mut BDD<TerminalId>, tokens: &mut I) -> Option<BDDFunc>
where
    I: Iterator<Item = &'a Token>,
{
    let mut current_gate: Option<Gate> = None;
    let mut current_bdd_func: Option<BDDFunc> = None;
    while let Some(token) = tokens.next() {
        match token {
            Token::OpeningParenthesis => current_bdd_func = build_bdd_from_iter(bdd, tokens),
            Token::ClosingParenthesis => return current_bdd_func,
            Token::Terminal(c) => {
                match (current_bdd_func, current_gate) {
                    (None, None) => current_bdd_func = Some(bdd.terminal(*c)),
                    (Some(bdd_func), Some(gate)) => {
                        let this_bdd_func = bdd.terminal(*c); // add this new terminal
                        match gate {
                            Gate::And => current_bdd_func = Some(bdd.and(bdd_func, this_bdd_func)),
                            Gate::Or => current_bdd_func = Some(bdd.or(bdd_func, this_bdd_func)),
                            Gate::Nand => {
                                let tmp_bdd_func = bdd.and(bdd_func, this_bdd_func);
                                current_bdd_func = Some(bdd.not(tmp_bdd_func));
                            }
                            Gate::Nor => {
                                let tmp_bdd_func = bdd.or(bdd_func, this_bdd_func);
                                current_bdd_func = Some(bdd.not(tmp_bdd_func));
                            }
                            Gate::Xor => current_bdd_func = Some(bdd.xor(bdd_func, this_bdd_func)),
                        }
                        current_gate = None;
                    }
                    _ => unreachable!("if tokens are properly parsed"),
                }
            }
            Token::Gate(gate) => match (current_bdd_func, current_gate) {
                (Some(_), None) => current_gate = Some(*gate),
                _ => unreachable!("if tokens are properly parsed"),
            },
            Token::Whitespace => unreachable!("if tokens are properly parsed"),
        }
    }
    current_bdd_func
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn build_bdd_single_terminal() {
        let mut bdd = BDD::<TerminalId>::new();
        let tokens = ParsedTokens::from_str("x").unwrap();
        let bdd_func = build_bdd(&mut bdd, &tokens).unwrap();
        assert_eq!(bdd.labels(), &['x']);

        let mut evals = HashMap::new();
        evals.insert('x', true);
        assert!(bdd.evaluate(bdd_func, &evals));
        let x_bool = evals.get_mut(&'x').unwrap();
        *x_bool = false;
        assert!(!bdd.evaluate(bdd_func, &evals));
    }

    #[test]
    fn build_bdd_basic_and() {
        let mut bdd = BDD::<TerminalId>::new();
        let tokens = ParsedTokens::from_str("a AND b").unwrap();
        let bdd_func = build_bdd(&mut bdd, &tokens).unwrap();
        // NOTE bdd.labels() returns labels in a random order
        assert_eq!(bdd.labels().len(), 2);
        assert!(bdd.labels().contains(&'a'));
        assert!(bdd.labels().contains(&'b'));

        let mut evals = HashMap::new();
        evals.insert('a', true);
        evals.insert('b', true);
        assert!(bdd.evaluate(bdd_func, &evals));
        let a_bool = evals.get_mut(&'a').unwrap();
        *a_bool = false;
        assert!(!bdd.evaluate(bdd_func, &evals));
    }

    #[test]
    fn build_bdd_basic_or() {
        let mut bdd = BDD::<TerminalId>::new();
        let tokens = ParsedTokens::from_str("d   ORe").unwrap();
        let bdd_func = build_bdd(&mut bdd, &tokens).unwrap();
        // NOTE bdd.labels() returns labels in a random order
        assert_eq!(bdd.labels().len(), 2);
        assert!(bdd.labels().contains(&'d'));
        assert!(bdd.labels().contains(&'e'));

        let mut evals = HashMap::new();
        evals.insert('d', true);
        evals.insert('e', true);
        assert!(bdd.evaluate(bdd_func, &evals));
        let e_bool = evals.get_mut(&'e').unwrap();
        *e_bool = false;
        assert!(bdd.evaluate(bdd_func, &evals));
        let d_bool = evals.get_mut(&'d').unwrap();
        *d_bool = false;
        assert!(!bdd.evaluate(bdd_func, &evals)); // both false
    }

    #[test]
    fn build_bdd_compound() {
        let mut bdd = BDD::<TerminalId>::new();
        let tokens = ParsedTokens::from_str("a AND b OR ((a NAND d) OR e)").unwrap();
        let bdd_func = build_bdd(&mut bdd, &tokens).unwrap();
        // NOTE bdd.labels() returns labels in a random order
        assert_eq!(bdd.labels().len(), 4);
        assert!(bdd.labels().contains(&'a'));
        assert!(bdd.labels().contains(&'b'));
        assert!(bdd.labels().contains(&'d'));
        assert!(bdd.labels().contains(&'e'));

        let mut evals = HashMap::new();
        evals.insert('a', true);
        evals.insert('b', true);
        evals.insert('d', true);
        evals.insert('e', true);
        // x = a && b = true
        // y = a !& d = false
        // z = y || e = true
        // x || z = true
        assert!(bdd.evaluate(bdd_func, &evals));
        let b_bool = evals.get_mut(&'b').unwrap();
        *b_bool = false;
        // x = a && b = false
        // y = a !& d = false
        // z = y || e = true
        // x || z = true
        assert!(bdd.evaluate(bdd_func, &evals));
        let e_bool = evals.get_mut(&'e').unwrap();
        *e_bool = false;
        // x = a && b = false
        // y = a !& d = false
        // z = y || e = false
        // x || z = false
        assert!(!bdd.evaluate(bdd_func, &evals));
        let d_bool = evals.get_mut(&'d').unwrap();
        *d_bool = false;
        // x = a && b = false
        // y = a !& d = true
        // z = y || e = true
        // x || z = true
        assert!(bdd.evaluate(bdd_func, &evals));
        let a_bool = evals.get_mut(&'a').unwrap();
        *a_bool = false;
        // x = a && b = false
        // y = a !& d = true
        // z = y || e = true
        // x || z = true
        assert!(bdd.evaluate(bdd_func, &evals));
    }
}
