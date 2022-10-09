use crate::gate::Gate;
use crate::token::TokenTree;
use crate::TerminalId;

use boolean_expression::{BDDFunc, BDD};

/// Builds a BDD (Binary Decision Diagram) from a parsed [TokenTree].
///
/// Works with (almost arbitrary) compound logic, e.g. `"((a OR b) OR (c AND
/// d)) XOR a"`.
pub fn build_bdd(bdd: &mut BDD<TerminalId>, token_tree: TokenTree) -> BDDFunc {
    match token_tree {
        TokenTree::Terminal(c) => bdd.terminal(c),
        TokenTree::Gate { gate, left, right } => {
            let left_bdd_func = build_bdd(bdd, *left);
            let right_bdd_func = build_bdd(bdd, *right);
            match gate {
                Gate::And => bdd.and(left_bdd_func, right_bdd_func),
                Gate::Or => bdd.or(left_bdd_func, right_bdd_func),
                Gate::Nand => {
                    let tmp_bdd_func = bdd.and(left_bdd_func, right_bdd_func);
                    bdd.not(tmp_bdd_func)
                }
                Gate::Nor => {
                    let tmp_bdd_func = bdd.or(left_bdd_func, right_bdd_func);
                    bdd.not(tmp_bdd_func)
                }
                Gate::Xor => bdd.xor(left_bdd_func, right_bdd_func),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn build_bdd_single_terminal() {
        let mut bdd = BDD::<TerminalId>::new();
        let tree = TokenTree::from_str("x").unwrap();
        let bdd_func = build_bdd(&mut bdd, tree);
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
        let tree = TokenTree::from_str("a AND b").unwrap();
        let bdd_func = build_bdd(&mut bdd, tree);
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
        let tree = TokenTree::from_str("d   ORe").unwrap();
        let bdd_func = build_bdd(&mut bdd, tree);
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
        let tree = TokenTree::from_str("a AND b OR ((a NAND d) OR e)").unwrap();
        let bdd_func = build_bdd(&mut bdd, tree);
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
