use anyhow::Result;
use std::str::FromStr;

use crate::gate::Gate;
use crate::token::TokenTree;
use crate::TerminalId;

use boolean_expression::{BDDFunc, BDD};

pub struct BDDData {
    pub bdd: BDD<TerminalId>,
    pub root_func: BDDFunc,
}

impl BDDData {
    /// Builds a BDD (Binary Decision Diagram) from a parsed [TokenTree].
    ///
    /// Works with (almost arbitrary) compound logic, e.g. `"((a OR b) OR (c AND
    /// d)) XOR a"`.
    pub fn build_bdd_from_source(source: &str) -> Result<Self> {
        let tree = TokenTree::from_str(source)?;
        let mut bdd = BDD::<TerminalId>::new();
        let root_func = Self::build_bdd(&mut bdd, tree);
        Ok(BDDData { bdd, root_func })
    }

    fn build_bdd(bdd: &mut BDD<TerminalId>, token_tree: TokenTree) -> BDDFunc {
        match token_tree {
            TokenTree::Terminal(c) => bdd.terminal(c),
            TokenTree::Gate { gate, left, right } => {
                let left_bdd_func = Self::build_bdd(bdd, *left);
                let right_bdd_func = Self::build_bdd(bdd, *right);
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
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn build_bdd_single_terminal() {
        let bdd_data = BDDData::build_bdd_from_source("0").unwrap();
        let bdd = bdd_data.bdd;
        let bdd_func = bdd_data.root_func;

        assert_eq!(bdd.labels(), &[0]);

        let mut evals = HashMap::new();
        evals.insert(0, true);
        assert!(bdd.evaluate(bdd_func, &evals));
        let x_bool = evals.get_mut(&0).unwrap();
        *x_bool = false;
        assert!(!bdd.evaluate(bdd_func, &evals));
    }

    #[test]
    fn build_bdd_basic_and() {
        let bdd_data = BDDData::build_bdd_from_source("111 AND 222").unwrap();
        let bdd = bdd_data.bdd;
        let bdd_func = bdd_data.root_func;

        // NOTE bdd.labels() returns labels in a random order
        assert_eq!(bdd.labels().len(), 2);
        assert!(bdd.labels().contains(&111));
        assert!(bdd.labels().contains(&222));

        let mut evals = HashMap::new();
        evals.insert(111, true);
        evals.insert(222, true);
        assert!(bdd.evaluate(bdd_func, &evals));
        let a_bool = evals.get_mut(&111).unwrap();
        *a_bool = false;
        assert!(!bdd.evaluate(bdd_func, &evals));
    }

    #[test]
    fn build_bdd_basic_or() {
        let bdd_data = BDDData::build_bdd_from_source("999   OR1000").unwrap();
        let bdd = bdd_data.bdd;
        let bdd_func = bdd_data.root_func;

        // NOTE bdd.labels() returns labels in a random order
        assert_eq!(bdd.labels().len(), 2);
        assert!(bdd.labels().contains(&999));
        assert!(bdd.labels().contains(&1000));

        let mut evals = HashMap::new();
        evals.insert(999, true);
        evals.insert(1000, true);
        assert!(bdd.evaluate(bdd_func, &evals));
        let e_bool = evals.get_mut(&999).unwrap();
        *e_bool = false;
        assert!(bdd.evaluate(bdd_func, &evals));
        let d_bool = evals.get_mut(&1000).unwrap();
        *d_bool = false;
        assert!(!bdd.evaluate(bdd_func, &evals)); // both false
    }

    #[test]
    fn build_bdd_compound() {
        let bdd_data = BDDData::build_bdd_from_source("0 AND 1 OR ((0 NAND 3) OR 4)").unwrap();
        let bdd = bdd_data.bdd;
        let bdd_func = bdd_data.root_func;
        // NOTE bdd.labels() returns labels in a random order
        assert_eq!(bdd.labels().len(), 4);
        assert!(bdd.labels().contains(&0));
        assert!(bdd.labels().contains(&1));
        assert!(bdd.labels().contains(&3));
        assert!(bdd.labels().contains(&4));

        let mut evals = HashMap::new();
        evals.insert(0, true);
        evals.insert(1, true);
        evals.insert(3, true);
        evals.insert(4, true);
        // x = a && b = true
        // y = a !& d = false
        // z = y || e = true
        // x || z = true
        assert!(bdd.evaluate(bdd_func, &evals));
        let b_bool = evals.get_mut(&1).unwrap();
        *b_bool = false;
        // x = a && b = false
        // y = a !& d = false
        // z = y || e = true
        // x || z = true
        assert!(bdd.evaluate(bdd_func, &evals));
        let e_bool = evals.get_mut(&4).unwrap();
        *e_bool = false;
        // x = a && b = false
        // y = a !& d = false
        // z = y || e = false
        // x || z = false
        assert!(!bdd.evaluate(bdd_func, &evals));
        let d_bool = evals.get_mut(&3).unwrap();
        *d_bool = false;
        // x = a && b = false
        // y = a !& d = true
        // z = y || e = true
        // x || z = true
        assert!(bdd.evaluate(bdd_func, &evals));
        let a_bool = evals.get_mut(&0).unwrap();
        *a_bool = false;
        // x = a && b = false
        // y = a !& d = true
        // z = y || e = true
        // x || z = true
        assert!(bdd.evaluate(bdd_func, &evals));
    }
}
